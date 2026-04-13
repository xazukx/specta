use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use crate::{Constants, NamedConstant, ResolvedTypes, Types};

use super::{
    ExportLanguage, ImportInfo, filesystem,
    layout::{IndexFileConfig, Layout},
    module_graph::{Module, build_module_graph},
    path_resolver::PathResolver,
};

/// Information collected during index file generation about each generated file.
struct FileExportInfo {
    /// The exported type names in this file.
    exported_names: Vec<String>,
    /// The module path this file corresponds to.
    module_path: String,
}

/// The main orchestrator for file-based export.
///
/// Handles module graph construction, filesystem layout, path resolution,
/// file writing, and stale file cleanup. Delegates language-specific
/// rendering to the [`ExportLanguage`] trait.
pub struct ExportPipeline;

impl ExportPipeline {
    /// Export types to a directory using the given language exporter and layout.
    ///
    /// For `Layout::FlatFile` and `Layout::ModulePrefixedName`, `path` is a file path.
    /// For `Layout::Files`, `path` is a directory.
    pub fn export_to<L: ExportLanguage>(
        lang: &L,
        layout: &Layout,
        path: &Path,
        resolved: &ResolvedTypes,
    ) -> Result<(), L::Error> {
        match layout {
            Layout::SingleFile(_) => {
                let result = Self::export_to_string(lang, layout, resolved)?;
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, result)?;
                Ok(())
            }
            Layout::MultiFile(config) => {
                let types = resolved.as_types();
                let constants = resolved.constants();
                let mut files: HashMap<PathBuf, String> = HashMap::new();
                let mut file_info: HashMap<PathBuf, FileExportInfo> = HashMap::new();
                let path_resolver = PathResolver::new(
                    lang.file_extension(),
                    lang.index_file_stem(),
                    layout,
                    types,
                    constants,
                );

                let mut module_graph = build_module_graph(types, constants);
                let root_constants: Vec<_> = std::mem::take(&mut module_graph.constants);

                let mut root_types = String::new();
                let mut root_exports = HashMap::new();
                Self::export_module(
                    lang,
                    types,
                    &mut module_graph,
                    &mut root_types,
                    &mut root_exports,
                    path,
                    &mut files,
                    &mut file_info,
                    &path_resolver,
                )?;

                // Build index/root file
                let has_root_content = !root_types.is_empty() || !root_constants.is_empty();
                if has_root_content {
                    let index_stem = lang.index_file_stem().unwrap_or("index");
                    let mut index_path = path.join(index_stem);
                    index_path.set_extension(lang.file_extension());

                    let mut out = lang.render_file_header();

                    if !root_types.is_empty() {
                        if !out.is_empty() {
                            out.push('\n');
                        }
                        out.push_str(&root_types);
                    }

                    if !root_constants.is_empty() {
                        let const_refs: Vec<&NamedConstant> =
                            root_constants.iter().copied().collect();
                        let rendered = lang.render_constants(&const_refs)?;
                        if !rendered.is_empty() {
                            out.push('\n');
                            out.push_str(&rendered);
                        }
                    }

                    files.insert(index_path, out);
                }

                // Generate index files if configured
                if let IndexFileConfig::ReExportAll = &config.index_files {
                    if lang.index_file_stem().is_some() {
                        Self::generate_index_files(
                            lang,
                            path,
                            &file_info,
                            &mut files,
                            &path_resolver,
                        )?;
                    }
                }

                // Clean up existing directory if it's a file
                match path.metadata() {
                    Ok(meta) if !meta.is_dir() => {
                        std::fs::remove_file(path).or_else(|err| {
                            if err.kind() == std::io::ErrorKind::NotFound {
                                Ok(())
                            } else {
                                Err(err)
                            }
                        })?;
                    }
                    Ok(_) => {}
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                    Err(err) => return Err(err.into()),
                }

                // Write all files
                for (file_path, content) in &files {
                    file_path
                        .parent()
                        .map(std::fs::create_dir_all)
                        .transpose()?;
                    std::fs::write(file_path, content)?;
                }

                // Clean up stale files
                let extensions = lang.stale_file_extensions();
                let ext_refs: Vec<&str> = extensions.iter().map(|s| *s).collect();
                filesystem::cleanup_stale_files(path, &files, &ext_refs, lang.generated_marker())?;

                Ok(())
            }
        }
    }

    /// Export types to a single string (for `FlatFile` and `ModulePrefixedName` layouts).
    pub fn export_to_string<L: ExportLanguage>(
        lang: &L,
        _layout: &Layout,
        resolved: &ResolvedTypes,
    ) -> Result<String, L::Error> {
        let types = resolved.as_types();
        let mut out = lang.render_file_header();

        // Collect all types, sorted
        let ndts: Vec<&_> = types
            .into_sorted_iter()
            .filter(|ndt| ndt.requires_reference(types))
            .collect();

        if !ndts.is_empty() {
            let result = lang.render_module_types(types, &ndts, "")?;
            if !result.body.is_empty() {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&result.body);
            }
        }

        // Constants
        let constants = resolved.constants();
        if !constants.is_empty() {
            let const_refs: Vec<&NamedConstant> = constants.iter().collect();
            let rendered = lang.render_constants(&const_refs)?;
            if !rendered.is_empty() {
                out.push('\n');
                out.push_str(&rendered);
            }
        }

        Ok(out)
    }

    /// Build a PathResolver for the given types and layout without writing files.
    pub fn build_path_resolver<L: ExportLanguage>(
        lang: &L,
        layout: &Layout,
        types: &Types,
        constants: &Constants,
    ) -> PathResolver {
        PathResolver::new(
            lang.file_extension(),
            lang.index_file_stem(),
            layout,
            types,
            constants,
        )
    }

    /// Recursively export a module and its children.
    ///
    /// `root_path` is always the top-level output directory (not recursively built up).
    /// File placement is determined by PathResolver, which respects FolderGrouping.
    fn export_module<L: ExportLanguage>(
        lang: &L,
        types: &Types,
        module: &mut Module<'_>,
        s: &mut String,
        root_exports: &mut HashMap<String, std::panic::Location<'static>>,
        root_path: &Path,
        files: &mut HashMap<PathBuf, String>,
        file_info: &mut HashMap<PathBuf, FileExportInfo>,
        path_resolver: &PathResolver,
    ) -> Result<bool, L::Error> {
        module.types.sort_by(|a, b| {
            a.name()
                .cmp(b.name())
                .then(a.module_path().cmp(b.module_path()))
                .then(a.location().cmp(&b.location()))
        });

        let module_types: Vec<&_> = module
            .types
            .iter()
            .filter(|ndt| ndt.requires_reference(types))
            .copied()
            .collect();

        let result = lang.render_module_types(types, &module_types, "")?;
        let rendered_types = result.body;
        let exports = result.exports;
        let referenced_modules = result.referenced_modules;

        // Generate imports (filter out self-references)
        let import_paths: BTreeMap<String, ImportInfo> = referenced_modules
            .into_iter()
            .filter(|(module_path, _)| module_path != module.module_path.as_ref())
            .collect();

        if !import_paths.is_empty() {
            let import_block =
                lang.render_imports(module.module_path.as_ref(), &import_paths, path_resolver)?;
            if !import_block.is_empty() {
                s.push('\n');
                s.push_str(&import_block);
            }
        }

        if !import_paths.is_empty() && !rendered_types.is_empty() {
            s.push('\n');
        }

        s.push_str(&rendered_types);

        // Constants belonging to this module
        if !module.constants.is_empty() {
            module.constants.sort_by(|a, b| a.name.cmp(&b.name));
            let const_refs: Vec<&NamedConstant> = module.constants.iter().copied().collect();
            let rendered = lang.render_constants(&const_refs)?;
            if !rendered.is_empty() {
                if !rendered_types.is_empty() {
                    s.push('\n');
                }
                s.push_str(&rendered);
            }
        }

        let has_content = !exports.is_empty() || !module.constants.is_empty();
        root_exports.extend(exports.clone());

        // Process child modules
        for (_name, child_module) in &mut module.children {
            if child_module.types.is_empty()
                && child_module.constants.is_empty()
                && child_module.children.is_empty()
            {
                continue;
            }

            let mut out = lang.render_file_header();
            let mut child_exports = HashMap::new();

            let has_types = Self::export_module(
                lang,
                types,
                child_module,
                &mut out,
                &mut child_exports,
                root_path,
                files,
                file_info,
                path_resolver,
            )?;

            if has_types {
                // Use PathResolver for correct file placement (respects FolderGrouping)
                let child_path = path_resolver
                    .module_file_path(child_module.module_path.as_ref())
                    .map(|rel| root_path.join(rel))
                    .unwrap_or_else(|| {
                        let mut p = root_path.join(child_module.module_path.replace("::", "/"));
                        p.set_extension(lang.file_extension());
                        p
                    });

                let export_names: Vec<String> = child_exports.keys().cloned().collect();
                file_info.insert(
                    child_path.clone(),
                    FileExportInfo {
                        exported_names: export_names,
                        module_path: child_module.module_path.to_string(),
                    },
                );

                files.insert(child_path, out);
            }
        }

        Ok(has_content)
    }

    /// Generate index files for directories that contain generated files.
    fn generate_index_files<L: ExportLanguage>(
        lang: &L,
        _root: &Path,
        file_info: &HashMap<PathBuf, FileExportInfo>,
        files: &mut HashMap<PathBuf, String>,
        path_resolver: &PathResolver,
    ) -> Result<(), L::Error> {
        use super::IndexFileEntry;

        // Group files by their parent directory
        let mut dirs: BTreeMap<PathBuf, Vec<(&PathBuf, &FileExportInfo)>> = BTreeMap::new();
        for (path, info) in file_info {
            if let Some(parent) = path.parent() {
                dirs.entry(parent.to_path_buf())
                    .or_default()
                    .push((path, info));
            }
        }

        let index_stem = match lang.index_file_stem() {
            Some(stem) => stem,
            None => return Ok(()),
        };

        for (dir, dir_files) in &dirs {
            let entries: Vec<IndexFileEntry> = dir_files
                .iter()
                .filter_map(|(path, info)| {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .filter(|stem| *stem != index_stem)
                        .map(|stem| IndexFileEntry {
                            file_stem: stem,
                            exported_names: &info.exported_names,
                            module_path: &info.module_path,
                        })
                })
                .collect();

            // Find subdirectories that have their own index files
            let subdirs: Vec<&str> = dirs
                .keys()
                .filter(|other_dir| other_dir.parent() == Some(dir) && *other_dir != dir)
                .filter_map(|d| d.file_name().and_then(|n| n.to_str()))
                .collect();

            if !entries.is_empty() || !subdirs.is_empty() {
                let mut index_path = dir.join(index_stem);
                index_path.set_extension(lang.file_extension());

                let rendered = lang.render_index_file(&entries, &subdirs, path_resolver)?;
                if rendered.is_empty() {
                    continue;
                }

                // Append re-exports to existing index file, or create a new one
                if let Some(existing) = files.get_mut(&index_path) {
                    if !existing.is_empty() {
                        existing.push('\n');
                    }
                    existing.push_str(&rendered);
                } else {
                    let mut out = lang.render_file_header();
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&rendered);
                    files.insert(index_path, out);
                }
            }
        }

        Ok(())
    }
}
