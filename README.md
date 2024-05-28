# Editor
Trying to make a 'mini' vim like editor in rust. Currently a work in progress.

# Test Branch
This branch is functionally different than main:
    - better motions
    - command buffer
    - ability to use multiple buffers
    - etc

## Todo
* impl ropes into system
    * impl Ropey
        * impl movement in buffer
        * buffer saving and other actions
* improve directory buffer events
    * added better pathing for nesting dirs
* improve error handling
* clearing sys messages
* add syntax highlighting with tree-sitter
    * add basic color schemes
* work out messages not displaying and clearing
    * due to clearing message when changing mode
* define runtime better for action (motions, commands, etc) handling
* improve/fix testing suite
* fix all todos and fixes in proj
* add actions, like delete, to motions
* later implement ptr_x for longer lines
* add functionality to file tree
    * refactor functions and system
* add f/F/t/T
* ~improve commands ui
