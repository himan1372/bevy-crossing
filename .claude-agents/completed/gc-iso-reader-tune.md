# gc-iso-reader-tune — fix the maker field; add project lookups

Philip: "fix it and make it efficient for you and this project".

- [x] info: maker code is header bytes 4-5 (was 6-8).
- [x] Read Yaz0 (.szs) and RARC (.arc) files on the disc (foresta.rel.szs, forest_1st.arc).
- [x] sym: find symbols in disc/data/*.sym and hexdump their bytes from the REL/DOL (stdout only, nothing written).
- [x] Short, filterable output; tests extended (synthetic data only).
