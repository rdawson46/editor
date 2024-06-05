# Editor
Trying to make a 'mini' vim like editor in rust. Currently a work in progress.

# Goal
Make an editor that:
* is a usable simple text editor
* configurable
* customizable

## Todo
* get tree sitter working
    * impl highlighting in ui.rs
* add syntax highlighting with tree-sitter
    * add basic color schemes
* overflow on word jumps, screen moving
* windows from multi buffer view
* baby git integration
* improve screen moving
* improve error handling
* work out messages not displaying and clearing
    * due to clearing message when changing mode
* define runtime better for action (motions, commands, etc) handling
* arrow key movement for buffer and command line
* improve/fix testing suite
* fix all todos and fixes in proj
* add actions, like delete, to motions
* later implement ptr_x for longer lines
* add functionality to file tree
    * refactor functions and system
* add f/F/t/T
* ~improve commands ui
