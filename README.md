# Configuration
`datestamp_files` is a command line tool that renames the file or files given, to `YYYY-MM-DD - <filename>`.
It's pretty darn simple, but its fast (local) and powerful (obsidian).

If you run it in local mode (just changing and local file or directory, with recursive optionally), it takes a `-f` or `-d` argument. For  recursive `-rd`.

To run it in obsidian mode, you will need to:
1. Install the Obsidian Local REST API
2. Install the [Obsidian Local REST API Rename Endpoint](https://github.com/mtuckerb/obsidian-local-rest-api-rename)
3. create a `.env` file with at least the following lines
```.env
OBSIDIAN_API_URL=https://localhost:27124
OBSIDIAN_API_KEY=<the key provided by Obsidian Local Rest API plugin>
```

Then call `datestamp_files -o -d <Diretory relative to your vault root>` 
or `datestamp_files -o -d <file path relative to your vault root>`

When used this way, Obsidian's API is ultimately responsible for renaming the file, which triggers it's cache update for all links to that file.
This means that you won't end up with dead links after the rename. 


# ToDo
Might be fun to make this more extensible, so it can rename files based on any search terms… Might be good to make a DSL for the new filename too ¯\_(ツ)_/¯


# NixOS
If you're cool you can add this to your nix system by adding `datestamp-files.url = "github:mtuckerb/datestamp_files";` to your flake input and then 
