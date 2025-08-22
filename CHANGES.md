* 0.1.6
    * New `set` command that sets context values for the template
    * New `include` command that includes another echo file
* 0.1.5
    * New `command` command that prints to the command line of the editor
    * New `command_clear_timeout` command that clears the command line after a while
    * `linepause` and `line_pause` both works
    * `closepopup` and `close_popup` both works
* 0.1.4
    * New `write` command that writes the buffer to disk
    * BUGFIX: newline chars didn't render if they were the last instructions
* 0.1.3
    * BUGFIX: instructions per second did not respect `sleep` instructions
* 0.1.2
    * BUGFIX: instructions per second did not respect `wait` instructions
* 0.1.1
    * Speed now equates to "instructions per second"
    * Optimisations
* 0.1.0
    * First release
