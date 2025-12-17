# TODO items

### Import and update from source

#### Exporting projects

Currently the export options are Export and Export as template. Update this to,

 - Export ZIP
 - Export to filesystem
 - Export as template

Export to filesystem is the same as Export ZIP, but exports the project to a directory provided by the user (create if not exists)

#### Importing projects

The Import Project button currently imports an exported project ZIP file.
Make this two options

 - Import (ZIP)
 - Import (Filesystem)

When importing from filesystem, add a toggle "Keep connection". If this is set, the import source directory path is kept as an attribute in the Project data and on the Project page next to "Export Project" there will now be a button "Re-import from source", which resets the project, and a button "Re-export to source" which exports the project to the connected directory (when exporting to a connected directory, do not touch dotfiles (example: .git) and make sure that the export also removes files that have not been written but would have been written by a previous export so represent deleted or changed assets like plans)


