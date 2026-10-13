# iso reader

Read ISO 9660 disk images on Windows and Linux without mounting them. Use the bundled Python helper for listing contents, inspecting volume metadata, and extracting files. Fall back to 7-Zip or native OS mounting when Python is unavailable.

## Read an ISO with the helper script

- Install pycdlib once with `pip install pycdlib`. It is pure Python and works identically on Windows, Linux, and macOS with no admin rights and no mounting.

- Run the helper at `.claude/skills/iso-reader/scripts/iso_tool.py` from the repo root. Call it with `python` on Windows or `python3` on Linux.

- Show volume metadata with `python .claude/skills/iso-reader/scripts/iso_tool.py info image.iso`. This prints the volume label, system id, block count, block size, and total bytes.

- List everything inside with `python .claude/skills/iso-reader/scripts/iso_tool.py list image.iso`. Add `--dir /DOCS` to list one directory. Paths inside the ISO use forward slashes on both platforms.

- Extract one file with `python .claude/skills/iso-reader/scripts/iso_tool.py extract image.iso /README.TXT out.txt`. Find the exact in-ISO path from the list output first.

- Extract the whole image with `python .claude/skills/iso-reader/scripts/iso_tool.py extract-all image.iso outdir`. This recreates the directory tree under `outdir`.

- Treat a `PyCdlibInvalidISO` error as a corrupt or non-ISO file. The script already reports a missing input file cleanly.

## Windows alternatives without Python

- Use 7-Zip from the command line when it is installed. List with `"C:\Program Files\7-Zip\7z.exe" l image.iso` and extract with `"C:\Program Files\7-Zip\7z.exe" x image.iso -oC:\out`.

- Mount natively in PowerShell with `Mount-DiskImage -ImagePath C:\path\image.iso`. The ISO appears as a new drive letter. Release it with `Dismount-DiskImage -ImagePath C:\path\image.iso`.

- Prefer the pycdlib helper over mounting when you only need to read or extract. Mounting is slower and leaves a drive letter behind if the dismount is forgotten.

## Linux alternatives without Python

- Use p7zip when it is installed. List with `7z l image.iso` and extract with `7z x image.iso -ooutdir`.

- Mount with `sudo mount -o loop,ro image.iso /mnt/iso` and unmount with `sudo umount /mnt/iso`. This needs root and a prepared mount point.

## Verified and untested

- The pycdlib helper was tested on Linux against a generated ISO. All four commands ran and extracted bytes matched the originals exactly.

- The 7-Zip and PowerShell commands are documented from their stable interfaces but were not executed on Windows in this environment. Confirm the 7-Zip install path on the target machine before scripting it.
