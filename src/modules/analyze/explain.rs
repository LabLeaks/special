/**
@module SPECIAL.MODULES.ANALYZE.EXPLAIN
Defines shared plain-language and exact explanation text for architecture analysis metrics so renderers can present the same meaning across language providers.
*/
// @fileimplements SPECIAL.MODULES.ANALYZE.EXPLAIN
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MetricExplanationKey {
    CyclomaticTotal,
    CyclomaticMax,
    CognitiveTotal,
    CognitiveMax,
    QualityPublicFunctions,
    QualityParameters,
    QualityBoolParameters,
    QualityRawStringParameters,
    QualityPanicSites,
    UnownedUnreachedItems,
    ConnectedItem,
    OutboundHeavyItem,
    IsolatedItem,
    UnreachedItem,
    UnreachedItems,
    DuplicateItems,
    HighestComplexityItem,
    ParameterHeavyItem,
    StringlyBoundaryItem,
    PanicHeavyItem,
    FanIn,
    FanOut,
    AfferentCoupling,
    EfferentCoupling,
    Instability,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MetricExplanation {
    pub plain: &'static str,
    pub precise: &'static str,
}

pub(crate) fn metric_explanation(key: MetricExplanationKey) -> MetricExplanation {
    match key {
        MetricExplanationKey::CyclomaticTotal => MetricExplanation {
            plain: "this is the combined branchiness across owned functions and methods.",
            precise: "sum of per-item cyclomatic complexity across analyzed owned implementation.",
        },
        MetricExplanationKey::CyclomaticMax => MetricExplanation {
            plain: "this is the branchiest single owned function or method.",
            precise: "maximum per-item cyclomatic complexity across analyzed owned implementation.",
        },
        MetricExplanationKey::CognitiveTotal => MetricExplanation {
            plain: "this is the combined reading and control-flow burden across owned functions and methods.",
            precise: "sum of per-item cognitive complexity across analyzed owned implementation.",
        },
        MetricExplanationKey::CognitiveMax => MetricExplanation {
            plain: "this is the hardest single owned function or method to follow structurally.",
            precise: "maximum per-item cognitive complexity across analyzed owned implementation.",
        },
        MetricExplanationKey::QualityPublicFunctions => MetricExplanation {
            plain: "this counts public entrypoints whose shape affects how others use this module.",
            precise: "count of analyzed owned public functions and methods.",
        },
        MetricExplanationKey::QualityParameters => MetricExplanation {
            plain: "this shows how much raw argument surface the module exposes through public APIs.",
            precise: "sum of typed parameters across analyzed owned public functions and methods.",
        },
        MetricExplanationKey::QualityBoolParameters => MetricExplanation {
            plain: "this highlights public APIs that steer behavior with on or off flags.",
            precise: "count of public parameters typed as bool across analyzed owned implementation.",
        },
        MetricExplanationKey::QualityRawStringParameters => MetricExplanation {
            plain: "this highlights public APIs that pass around raw string values instead of narrower named types.",
            precise: "count of public parameters typed as String, str, or Cow<str> across analyzed owned implementation.",
        },
        MetricExplanationKey::QualityPanicSites => MetricExplanation {
            plain: "this highlights places where recoverable runtime paths can still crash or abort abruptly.",
            precise: "count of panic-like macros and unwrap/expect method calls across analyzed owned implementation.",
        },
        MetricExplanationKey::UnownedUnreachedItems => MetricExplanation {
            plain: "this counts source items outside declared modules with no observed path from public or test roots, so they may be trapped, incidental, framework-driven, or unused.",
            precise: "count of unowned non-public, non-test items not reachable from public or test items through observed sibling call edges.",
        },
        MetricExplanationKey::ConnectedItem => MetricExplanation {
            plain: "this item is meaningfully tied into the rest of the module's owned implementation.",
            precise: "owned item with inbound or outbound references to other owned items in the same module.",
        },
        MetricExplanationKey::OutboundHeavyItem => MetricExplanation {
            plain: "this item reaches outward more than it talks to the rest of its own module.",
            precise: "owned item whose external references exceed its outbound references to sibling owned items.",
        },
        MetricExplanationKey::IsolatedItem => MetricExplanation {
            plain: "this item talks outward without much visible relationship to the rest of its own module.",
            precise: "owned item with zero inbound and outbound sibling references but at least one external reference.",
        },
        MetricExplanationKey::UnreachedItem => MetricExplanation {
            plain: "this private item has no observed path from public or test roots inside the analyzed implementation, so it may be trapped, incidental, or unused.",
            precise: "non-public, non-test owned item not reachable from public or test items through observed sibling call edges.",
        },
        MetricExplanationKey::UnreachedItems => MetricExplanation {
            plain: "this counts private owned items with no observed path from public or test roots, so they may be trapped, incidental, framework-driven, or unused.",
            precise: "count of non-public, non-test owned items not reachable from public or test items through observed sibling call edges.",
        },
        MetricExplanationKey::DuplicateItems => MetricExplanation {
            plain: "this counts owned items whose parser-normalized structure and substantive operation profile match another owned item, so they may indicate repeated implementation logic worth consolidating or reviewing.",
            precise: "count of analyzed owned items whose structural fingerprint and substantive call/control-flow profile match at least one other analyzed owned item.",
        },
        MetricExplanationKey::HighestComplexityItem => MetricExplanation {
            plain: "this is one of the most structurally complex owned items inside the module boundary.",
            precise: "owned item ranked by cognitive complexity, then cyclomatic complexity.",
        },
        MetricExplanationKey::ParameterHeavyItem => MetricExplanation {
            plain: "this item carries a relatively wide argument surface compared with its peers.",
            precise: "owned item ranked by parameter count, then raw string parameter count.",
        },
        MetricExplanationKey::StringlyBoundaryItem => MetricExplanation {
            plain: "this public item exposes raw string values at the module boundary.",
            precise: "owned public item ranked by raw string parameter count, then total parameter count.",
        },
        MetricExplanationKey::PanicHeavyItem => MetricExplanation {
            plain: "this item contains more crash-prone runtime sites than its peers.",
            precise: "owned item ranked by panic-like site count, then cognitive complexity.",
        },
        MetricExplanationKey::FanIn => MetricExplanation {
            plain: "other owned modules reach into this module.",
            precise: "distinct inbound concrete-module dependencies resolved from owned code.",
        },
        MetricExplanationKey::FanOut => MetricExplanation {
            plain: "this module reaches into other owned modules.",
            precise: "distinct outbound concrete-module dependencies resolved from owned code.",
        },
        MetricExplanationKey::AfferentCoupling => MetricExplanation {
            plain: "other owned modules depend on this module.",
            precise: "count of inbound concrete-module dependencies.",
        },
        MetricExplanationKey::EfferentCoupling => MetricExplanation {
            plain: "this module depends on other owned modules.",
            precise: "count of outbound concrete-module dependencies.",
        },
        MetricExplanationKey::Instability => MetricExplanation {
            plain: "this shows whether the module leans more on others than others lean on it.",
            precise: "efferent coupling / (afferent coupling + efferent coupling).",
        },
    }
}
