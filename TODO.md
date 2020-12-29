# Easy

 *  None of the doc comments are correct. The tests need to be rewritten.
*   There are vestigial methods from codespan and nom_locate. They need to be reviewed to see which
    ones make sense to keep, which can safely be removed, and which are missing.
 * Check that all crate features work as advertised.
 
# Medium

 * There is some unsafe code that shouldn't be required. Removing it might require rethinking the
  design.

# Hard

 * Determine proper trait constraints needed to make it generic over `SourceType`.
 * Bring back no-std support.
