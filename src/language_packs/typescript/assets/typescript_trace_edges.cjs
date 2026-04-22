const fs = require('node:fs');
const path = require('node:path');

function normalize(p) {
  const resolved = path.resolve(p);
  try {
    return fs.realpathSync.native(resolved).replace(/\\/g, '/');
  } catch {
    return resolved.replace(/\\/g, '/');
  }
}

function loadTypeScript(moduleEntry) {
  return require(moduleEntry);
}

function resolveImportedFiles(ts, sourceFile, options, host, trackedFiles) {
  const imported = ts.preProcessFile(sourceFile.text, true, true).importedFiles || [];
  const resolved = new Set();
  for (const entry of imported) {
    const moduleName = entry.fileName;
    const resolution = ts.resolveModuleName(
      moduleName,
      sourceFile.fileName,
      options,
      host
    );
    const resolvedFileName = resolution.resolvedModule && resolution.resolvedModule.resolvedFileName;
    if (!resolvedFileName) continue;
    const normalized = normalize(resolvedFileName);
    if (trackedFiles.has(normalized)) {
      resolved.add(normalized);
    }
  }
  return resolved;
}

function groupItemsByPath(items) {
  const grouped = new Map();
  for (const item of items) {
    const key = normalize(item.path);
    if (!grouped.has(key)) grouped.set(key, []);
    grouped.get(key).push(item);
  }
  for (const bucket of grouped.values()) {
    bucket.sort((a, b) => {
      const aSize = a.end_line - a.start_line;
      const bSize = b.end_line - b.start_line;
      return aSize - bSize || a.start_line - b.start_line;
    });
  }
  return grouped;
}

function findContainingItem(grouped, filePath, line) {
  const bucket = grouped.get(normalize(filePath));
  if (!bucket) return null;
  for (const item of bucket) {
    if (item.start_line <= line && line <= item.end_line) {
      return item;
    }
  }
  return null;
}

function declarationName(ts, decl) {
  if (decl.name && ts.isIdentifier(decl.name)) {
    return decl.name.text;
  }
  if (ts.isConstructorDeclaration(decl)) {
    const parent = decl.parent && decl.parent.parent;
    if (parent && ts.isClassDeclaration(parent) && parent.name) {
      return parent.name.text;
    }
  }
  return null;
}

function itemNameOffset(sourceFile, item) {
  const lineIndex = Math.max(0, item.start_line - 1);
  const lineStarts = sourceFile.getLineStarts();
  const lineStart = lineStarts[Math.min(lineIndex, lineStarts.length - 1)] || 0;
  const lineEnd =
    lineIndex + 1 < lineStarts.length ? lineStarts[lineIndex + 1] : sourceFile.end;
  const start = Math.min(lineStart + item.start_column, lineEnd);
  const lineText = sourceFile.text.slice(start, lineEnd);
  const offset = lineText.indexOf(item.name);
  if (offset >= 0) return start + offset;
  return sourceFile.getPositionOfLineAndCharacter(lineIndex, item.start_column);
}

function matchDeclaredItem(ts, grouped, decl) {
  const sourceFile = decl.getSourceFile();
  if (!sourceFile || sourceFile.isDeclarationFile) return null;
  const filePath = normalize(sourceFile.fileName);
  const line = sourceFile.getLineAndCharacterOfPosition(decl.getStart(sourceFile)).line + 1;
  const bucket = grouped.get(filePath);
  if (!bucket) return null;
  const name = declarationName(ts, decl);
  const candidates = bucket.filter((item) => {
    if (name && item.name !== name) return false;
    return item.start_line <= line && line <= item.end_line;
  });
  return candidates[0] || null;
}

function main() {
  const moduleEntry = process.argv[2];
  if (!moduleEntry) {
    process.stderr.write('typescript module entry is required\n');
    process.exit(1);
  }

  const ts = loadTypeScript(moduleEntry);
  const input = JSON.parse(fs.readFileSync(0, 'utf8'));
  const mode = input.mode || 'trace_edges';
  const grouped = groupItemsByPath(input.items);
  const trackedFiles = new Set(input.source_files.map(normalize));

  let rootNames = input.source_files.map(normalize);
  let options = {
    module: ts.ModuleKind.CommonJS,
    target: ts.ScriptTarget.ES2022,
    jsx: ts.JsxEmit.Preserve,
    allowJs: false,
    noEmit: true,
  };

  const configPath = ts.findConfigFile(input.root, ts.sys.fileExists);
  if (configPath) {
    const configFile = ts.readConfigFile(configPath, ts.sys.readFile);
    if (!configFile.error) {
      const parsed = ts.parseJsonConfigFileContent(
        configFile.config,
        ts.sys,
        path.dirname(configPath)
      );
      if (!parsed.errors.length) {
        rootNames = parsed.fileNames.map(normalize);
        options = parsed.options;
      }
    }
  }

  const languageServiceHost = {
    getScriptFileNames: () => rootNames,
    getScriptVersion: () => '0',
    getScriptSnapshot: (fileName) => {
      if (!ts.sys.fileExists(fileName)) return undefined;
      const text = ts.sys.readFile(fileName);
      return text === undefined ? undefined : ts.ScriptSnapshot.fromString(text);
    },
    getCurrentDirectory: () => input.root,
    getCompilationSettings: () => options,
    getDefaultLibFileName: (compilerOptions) => ts.getDefaultLibFilePath(compilerOptions),
    fileExists: ts.sys.fileExists,
    readFile: ts.sys.readFile,
    readDirectory: ts.sys.readDirectory,
    directoryExists: ts.sys.directoryExists && ts.sys.directoryExists.bind(ts.sys),
    getDirectories: ts.sys.getDirectories && ts.sys.getDirectories.bind(ts.sys),
  };
  const languageService = ts.createLanguageService(languageServiceHost);
  const program = languageService.getProgram() || ts.createProgram({ rootNames, options });
  const checker = program.getTypeChecker();

  if (mode === 'module_graph') {
    const fileEdgeSet = new Set();
    for (const sourceFile of program.getSourceFiles()) {
      if (sourceFile.isDeclarationFile) continue;
      const normalized = normalize(sourceFile.fileName);
      if (!trackedFiles.has(normalized)) continue;
      for (const importedFile of resolveImportedFiles(
        ts,
        sourceFile,
        options,
        languageServiceHost,
        trackedFiles
      )) {
        fileEdgeSet.add(`${normalized}\t${importedFile}`);
      }
    }
    const file_edges = Array.from(fileEdgeSet).map((entry) => {
      const [from, to] = entry.split('\t');
      return { from, to };
    });
    process.stdout.write(JSON.stringify({ file_edges }));
    return;
  }

  const edgeSet = new Set();

  function visit(sourceFile, node) {
    if (ts.isCallExpression(node)) {
      const callLine =
        sourceFile.getLineAndCharacterOfPosition(node.expression.getStart(sourceFile)).line + 1;
      const caller = findContainingItem(grouped, sourceFile.fileName, callLine);
      if (caller) {
        let decl = checker.getResolvedSignature(node)?.declaration || null;
        if (!decl) {
          const targetNode = ts.isPropertyAccessExpression(node.expression)
            ? node.expression.name
            : node.expression;
          const symbol = checker.getSymbolAtLocation(targetNode);
          if (symbol) {
            decl = symbol.valueDeclaration || (symbol.declarations && symbol.declarations[0]) || null;
          }
        }
        if (decl) {
          const callee = matchDeclaredItem(ts, grouped, decl);
          if (callee) {
            edgeSet.add(`${caller.stable_id}\t${callee.stable_id}`);
          }
        }
      }
    }
    ts.forEachChild(node, (child) => visit(sourceFile, child));
  }

  for (const sourceFile of program.getSourceFiles()) {
    if (sourceFile.isDeclarationFile) continue;
    if (!trackedFiles.has(normalize(sourceFile.fileName))) continue;
    visit(sourceFile, sourceFile);
  }

  for (const item of input.items) {
    const normalizedPath = normalize(item.path);
    if (!trackedFiles.has(normalizedPath)) continue;
    const sourceFile = program.getSourceFile(normalizedPath);
    if (!sourceFile) continue;
    const position = itemNameOffset(sourceFile, item);
    const referenceGroups = languageService.findReferences(normalizedPath, position) || [];
    for (const group of referenceGroups) {
      for (const reference of group.references || []) {
        if (reference.isDefinition) continue;
        const callerLine =
          sourceFile.fileName === reference.fileName
            ? sourceFile.getLineAndCharacterOfPosition(reference.textSpan.start).line + 1
            : (() => {
                const refSource = program.getSourceFile(reference.fileName);
                if (!refSource) return null;
                return refSource.getLineAndCharacterOfPosition(reference.textSpan.start).line + 1;
              })();
        if (callerLine == null) continue;
        const caller = findContainingItem(grouped, reference.fileName, callerLine);
        if (caller && caller.stable_id !== item.stable_id) {
          edgeSet.add(`${caller.stable_id}\t${item.stable_id}`);
        }
      }
    }
  }

  const edges = Array.from(edgeSet).map((entry) => {
    const [caller, callee] = entry.split('\t');
    return { caller, callee };
  });
  process.stdout.write(JSON.stringify({ edges }));
}

main();
