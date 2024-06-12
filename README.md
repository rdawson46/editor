# Editor
Trying to make a 'mini' vim like editor in rust. Currently a work in progress.

# Goal
Make an editor that:
* is a usable simple text editor
* configurable
* customizable

## Test
* Restablish Motions, KeyEvents, and event loop
* overflow on word jumps, screen moving
    * moving screen with jumps
* windows from multi buffer view
    * simply start with two per window
* terminal integration?
    * undecided on this one
* baby git integration
* improve error handling
* improve tree sitter working
    * improve syntax highlighting with tree-sitter
        * improve performance
        * add basic color schemes
* define runtime better for action (motions, commands, etc) handling
* arrow key movement for buffer and command line
* improve/fix testing suite
* fix all todos and fixes in proj
* add actions, like delete, to motions
* later implement ptr_x for longer lines
* add functionality to file tree
    * refactor functions and system
* add f/F/t/T
* mouse integration
* highlighting mode
* copy/paste
* ~improve commands ui
