/**
@group SPECIAL
special top-level structure.

@group SPECIAL.ROADMAP
Future planned extensions for special.

@group SPECIAL.VERIFICATION
Future verification-model work for special.

@group SPECIAL.QUALITY
special internal quality contracts.

@group SPECIAL.CONFIG
special configuration and root discovery.

@group SPECIAL.INIT
special project initialization workflow.
*/

/**
@spec SPECIAL.GROUPS
special supports structural group nodes that organize claims without making direct claims of their own.
*/

/**
@spec SPECIAL.GROUPS.STRUCTURAL_ONLY
special treats @group as structure-only and does not require verifies, attests, or planned markers on group nodes.
*/

/**
@spec SPECIAL.GROUPS.SPEC_MAY_HAVE_CHILDREN
special allows @spec nodes to have children while still remaining direct claims that need their own verifies, attests, or planned marker.
*/

/**
@spec SPECIAL.GROUPS.MUTUALLY_EXCLUSIVE
special does not allow the same node id to be declared as both @spec and @group.
*/
