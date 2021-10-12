# Easy

* There are vestigial methods from codespan and nom_locate. They need to be reviewed to
  see which ones make sense to keep, which can safely be removed, and which are missing.
* Check that all crate features work as advertised.
* Write more tests.
 
# Medium

*  There is some unsafe code that shouldn't be required.
   Removing it might require rethinking the design.

# Hard

 * Determine proper trait bounds needed to make it generic over `SourceType`.
 * Bring back no-std support.
