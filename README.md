# CL-Glyph

Define and render a glyph as centerlines

## Introduction

CL-Glyph is a DSL (domain-specific language) for describing a glyph as a collection of strokes. Each
stroke is defined by its thickness and centerline of an open _path_ or a closed _loop_. A centerline
is defined by its sequence of elements. An element is a rounded _node_, mitered _cusp_, or an _arc_
given as a combination of on/off-curve points and a radius.

This representation is human-readable, compact, and easily usable with a plotter, laser engraver, or
other type of CNC (computer numerical control) tool.

It was inspired by the shape diagrams in "Character Set for Optical Character Recognition (OCR-A)",
[ANSI X3.17-1981](https://nvlpubs.nist.gov/nistpubs/Legacy/FIPS/fipspub32-1-1981.pdf).
