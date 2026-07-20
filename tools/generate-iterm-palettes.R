#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly = TRUE)
source_root <- if (length(args) >= 1L) args[[1L]] else file.path("vendor", "ggsci")
output_file <- if (length(args) >= 2L) {
  args[[2L]]
} else {
  file.path("crates", "ggsci", "src", "generated", "iterm.rs")
}

palette_file <- file.path(source_root, "R", "palettes-iterm.R")
if (!file.exists(palette_file)) {
  stop("iTerm palette source not found: ", palette_file, call. = FALSE)
}

env <- new.env(parent = baseenv())
sys.source(palette_file, envir = env)
if (!exists("iterm_palettes", envir = env, inherits = FALSE)) {
  stop("iTerm palette source did not define iterm_palettes()", call. = FALSE)
}
if (!exists("ggsci_db_iterm", envir = env, inherits = FALSE)) {
  stop("iTerm palette source did not define ggsci_db_iterm", call. = FALSE)
}

theme_names <- get("iterm_palettes", envir = env, inherits = FALSE)()
database <- get("ggsci_db_iterm", envir = env, inherits = FALSE)
if (!is.character(theme_names) || anyNA(theme_names)) {
  stop("iterm_palettes() must return theme names", call. = FALSE)
}
if (!is.list(database) || is.null(names(database))) {
  stop("ggsci_db_iterm must be a named list", call. = FALSE)
}
if (!identical(theme_names, names(database))) {
  stop("iterm_palettes() ordering does not match ggsci_db_iterm", call. = FALSE)
}
if (anyDuplicated(theme_names)) {
  stop("duplicate canonical iTerm theme name: ", theme_names[[anyDuplicated(theme_names)]], call. = FALSE)
}

expected_theme_count <- 551L
expected_variants <- c("normal", "bright")
expected_channels <- c("Blue", "Yellow", "Red", "Cyan", "Green", "Magenta")
expected_colors_per_variant <- 6L
expected_total_color_count <- 6612L
hex_pattern <- "^#[0-9A-Fa-f]{6}$"

normalize_key <- function(value) {
  value <- trimws(value)
  chars <- strsplit(value, "", fixed = TRUE)[[1L]]
  normalized <- vapply(chars, function(ch) {
    if (ch %in% c("_", "-") || grepl("^[[:space:]]$", ch)) {
      " "
    } else {
      tolower(ch)
    }
  }, character(1L), USE.NAMES = FALSE)
  paste0(normalized, collapse = "")
}

normalized_names <- vapply(theme_names, normalize_key, character(1L), USE.NAMES = FALSE)
collision <- anyDuplicated(normalized_names)
if (collision) {
  first <- match(normalized_names[[collision]], normalized_names)
  stop(
    "normalized iTerm theme names collide: ",
    theme_names[[first]], " and ", theme_names[[collision]],
    call. = FALSE
  )
}

rust_string <- function(value) {
  chars <- strsplit(enc2utf8(value), "", fixed = TRUE)[[1L]]
  escaped <- vapply(chars, function(ch) {
    code <- utf8ToInt(ch)
    if (ch == "\\") return("\\\\")
    if (ch == "\"") return("\\\"")
    if (ch == "\n") return("\\n")
    if (ch == "\r") return("\\r")
    if (ch == "\t") return("\\t")
    if (code < 32L || code == 127L) return(sprintf("\\u{%X}", code))
    ch
  }, character(1L), USE.NAMES = FALSE)
  paste0("\"", paste0(escaped, collapse = ""), "\"")
}

rust_identifier <- function(value) {
  chars <- strsplit(enc2utf8(value), "", fixed = TRUE)[[1L]]
  encoded <- vapply(chars, function(ch) {
    if (grepl("^[A-Za-z0-9]$", ch)) {
      toupper(ch)
    } else {
      paste0("_U", sprintf("%04X", utf8ToInt(ch)), "_")
    }
  }, character(1L), USE.NAMES = FALSE)
  paste0("ITERM_", paste0(encoded, collapse = ""))
}

records <- vector("list", length(theme_names))
identifiers <- character(length(theme_names))
total_color_count <- 0L

for (index in seq_along(theme_names)) {
  theme <- theme_names[[index]]
  variants <- database[[theme]]
  if (!is.list(variants) || !identical(names(variants), expected_variants)) {
    stop(
      "iTerm theme must contain exactly normal then bright variants: ",
      theme,
      call. = FALSE
    )
  }

  colors <- vector("list", length(expected_variants))
  names(colors) <- expected_variants
  for (variant in expected_variants) {
    variant_colors <- variants[[variant]]
    if (!is.character(variant_colors) || length(variant_colors) != expected_colors_per_variant) {
      stop("iTerm variant must contain exactly six colors: ", theme, ":", variant, call. = FALSE)
    }
    if (!identical(names(variant_colors), expected_channels)) {
      stop("iTerm channels are not in canonical order: ", theme, ":", variant, call. = FALSE)
    }
    if (anyNA(variant_colors) || any(!grepl(hex_pattern, variant_colors))) {
      stop("iTerm variant contains invalid RGB hex colors: ", theme, ":", variant, call. = FALSE)
    }

    colors[[variant]] <- toupper(substring(variant_colors, 2L))
    total_color_count <- total_color_count + length(variant_colors)
  }

  identifier <- rust_identifier(theme)
  identifiers[[index]] <- identifier
  records[[index]] <- list(
    theme = theme,
    identifier = identifier,
    normal = colors[["normal"]],
    bright = colors[["bright"]]
  )
}

duplicate_identifier <- anyDuplicated(identifiers)
if (duplicate_identifier) {
  stop(
    "duplicate generated Rust identifier: ",
    identifiers[[duplicate_identifier]],
    call. = FALSE
  )
}

if (length(records) != expected_theme_count || total_color_count != expected_total_color_count) {
  stop(
    "unexpected iTerm counts; expected themes=", expected_theme_count,
    ", colors=", expected_total_color_count,
    "; got themes=", length(records),
    ", colors=", total_color_count,
    call. = FALSE
  )
}

lines <- c(
  "// Generated by tools/generate-iterm-palettes.R from ggsci/R/palettes-iterm.R.",
  "// Do not edit by hand.",
  "// Colors preserve the upstream Blue, Yellow, Red, Cyan, Green, Magenta order.",
  "#![allow(clippy::unreadable_literal)]",
  "",
  "use crate::{ItermPalette, Rgb};",
  "",
  paste0("pub(crate) const ITERM_PALETTE_COUNT: usize = ", length(records), ";"),
  paste0("pub(crate) const ITERM_VARIANT_COUNT: usize = ", length(expected_variants), ";"),
  paste0("pub(crate) const ITERM_COLORS_PER_VARIANT: usize = ", expected_colors_per_variant, ";"),
  paste0("pub(crate) const ITERM_TOTAL_COLOR_COUNT: usize = ", total_color_count, ";"),
  "pub(crate) const ITERM_DATA_SOURCE: &str = \"ggsci/R/palettes-iterm.R\";",
  ""
)

for (record in records) {
  lines <- c(
    lines,
    paste0("const ", record$identifier, "_NORMAL: &[Rgb; 6] = &["),
    paste0("    Rgb::from_hex(0x", record$normal, "),"),
    "];",
    paste0("const ", record$identifier, "_BRIGHT: &[Rgb; 6] = &["),
    paste0("    Rgb::from_hex(0x", record$bright, "),"),
    "];",
    ""
  )
}

lines <- c(lines, "pub(crate) static ITERM_PALETTES: &[ItermPalette] = &[")
for (record in records) {
  lines <- c(
    lines,
    paste0(
      "    ItermPalette::new(", rust_string(record$theme), ", ",
      record$identifier, "_NORMAL, ", record$identifier, "_BRIGHT),"
    )
  )
}
lines <- c(lines, "];", "")

dir.create(dirname(output_file), recursive = TRUE, showWarnings = FALSE)
writeLines(lines, output_file, useBytes = TRUE)

message(
  "generated ", length(records), " iTerm themes and ", total_color_count,
  " stored colors (", length(expected_variants), " variants x ",
  expected_colors_per_variant, " colors)"
)
