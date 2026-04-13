/**
@spec SPECIAL.VERIFICATION.COLOCATED_CONTRACTS
@planned
special makes verification contracts locally inspectable so a reviewer can validate the contract without chasing multiple files.
*/

/**
@spec SPECIAL.VERIFICATION.COLOCATED_CONTRACTS.SINGLE_REVIEW_SURFACE
@planned
special prefers verification designs where the important contract can be understood from one local review surface instead of being split across spec prose, helper code, and tool config.
*/

/**
@spec SPECIAL.VERIFICATION.DRIFT_WARNINGS
@planned
special warns when verification artifacts rely on external or ambient inputs that can silently drift.
*/

/**
@spec SPECIAL.VERIFICATION.DRIFT_WARNINGS.UNDECLARED_EXTERNAL_INPUTS
@planned
special warns when a verification artifact depends on undeclared external tools, configuration, environment variables, or shell features.
*/
