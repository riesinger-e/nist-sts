## template files

These files contain the templates for the template matching test for a specific bit length,
encoded as pure bytes, but byte-aligned with 0-padding. Files with more than 500 KiB size are
compressed with xz and have the `.xz` extension (this algorithm was chosen because it showed the best
compression ratio). These files were created by applying the script `convert_nist_templates` to the 
original template files from NIST.

To run the binary yourself (replace the template files with the originals): 
`cargo run -p scripts --bin convert_templates -- -i <NIST_TEMPLATE_DIR> -o <TEMPLATE_DIR>`

This format is much more space efficient than the original format. This allows 
embedding the template files directly into the library (3 MiB are not much compared
to the original 42 MiB). The bit length (and byte length) of each template is derived from
the name, e.g. `template2` &#x2192; 2 Bits &#x2192; 1 Byte.