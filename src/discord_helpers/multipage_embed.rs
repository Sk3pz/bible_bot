/*
 * This will be a module for creating embeds with multiple traversable pages.
 * Each page will be an embed, and users can navigate through them using buttons.
 * Some issues to overcome before creating this are:
 *  1. Every one of these will require storage of the state and the button interactions.
 *  2. The interaction handlers will be destroyed on restart, and persistance would mean more resources eaten up
 *
 * To solve these issues, maybe the interaction handlers can be cleaned up after a set time and the
 * enum can be changed to remove the buttons, putting a note in the footer that it is no longer interactive.
 */

// todo
