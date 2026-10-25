# Reverse Engineering Notes: MAT-File Format Specification (v5 & v7)

This document provides a clean-room specification of the binary interoperability format for MATLAB `.mat` files (specifically Level 5 and Level 7), derived from public specifications, format documentation, and binary layout verification.

---

## 1. Format Evolution & Overview

| Version | Format Architecture | Compression | Max File Size | Introduced |
| :--- | :--- | :--- | :--- | :--- |
| **Level 4** | Flat binary matrices | None | 2 GB | MATLAB 4 |
| **Level 5 (v5)** | Tagged hierarchical chunks | Optional zlib (deflate) | 2 GB | MATLAB 5 |
| **v7 / v7.0** | Level 5 with default per-variable zlib | zlib stream per variable | 2 GB | MATLAB 7.0 |
| **v7.3** | HDF5 container | HDF5 chunked gzip/deflate | Terabytes (64-bit) | MATLAB 7.3 |

`rustlab` targets Level 5 (v5/v7) for its native lightweight binary parser, with an optional HDF5 feature for v7.3.

---

## 2. Level 5 Binary Structure

A Level 5 MAT-file consists of a **128-byte Header** followed by a stream of **Tagged Data Elements**.

```
+-------------------------------------------------------------------------+
|                         128-Byte File Header                            |
|  [0..115]: Descriptive text (ASCII)                                     |
|  [116..123]: Subsystem data offset (8 bytes, int64)                     |
|  [124..125]: Version identifier (0x0100)                                |
|  [126..127]: Endian indicator ("IM" = Little-Endian, "MI" = Big-Endian) |
+-------------------------------------------------------------------------+
|                         Tagged Data Element 1                           |
|  - Tag (Type + Size)                                                    |
|  - Data Payload (Padded to 8-byte boundary)                             |
+-------------------------------------------------------------------------+
|                         Tagged Data Element 2 ...                       |
+-------------------------------------------------------------------------+
```

---

## 3. The 128-Byte File Header

1. **Header Text (Bytes 0–115)**:
   - 116 bytes of ASCII text containing descriptive information (e.g. `MATLAB 5.0 MAT-file, Platform: PCWIN64, Created on: ...`).
   - Unused trailing bytes are padded with spaces or null bytes.
2. **Subsystem Data Offset (Bytes 116–123)**:
   - 8 bytes integer. If subsystem data exists, contains file offset; otherwise all zeros.
3. **Version (Bytes 124–125)**:
   - 2-byte integer. Standard value is `0x0100` (Version 1.0 / Level 5).
4. **Endian Indicator (Bytes 126–127)**:
   - Contains the 2 ASCII characters `'M'` and `'I'`:
     - If bytes are `['I', 'M']` (`0x49, 0x4D`): **Little-Endian** host.
     - If bytes are `['M', 'I']` (`0x4D, 0x49`): **Big-Endian** host.

---

## 4. Tagged Data Elements

Each data record in the file is encapsulated in a tag.

### 4.1 Standard 8-Byte Tag
```
Bytes 0..3: Data Type (u32)
Bytes 4..7: Number of Bytes of data (u32)
Bytes 8..N: Data payload, padded with null bytes to a multiple of 8 bytes
```

### 4.2 Compressed Small Data Element (SDE)
If the data payload is **1 to 4 bytes**, MATLAB packs the tag and payload into a single 8-byte word:
```
Bytes 0..1: Data Type (u16)
Bytes 2..3: Number of Bytes (u16)
Bytes 4..7: Immediate Data Payload (1 to 4 bytes)
```
The parser detects an SDE if the upper 16 bits of the first 32-bit word are non-zero.

### 4.3 Data Type Identifiers
| ID (`u32`) | Constant | Description | Size |
| :--- | :--- | :--- | :--- |
| `1` | `miINT8` | 8-bit signed integer | 1 byte |
| `2` | `miUINT8` | 8-bit unsigned integer | 1 byte |
| `3` | `miINT16` | 16-bit signed integer | 2 bytes |
| `4` | `miUINT16` | 16-bit unsigned integer | 2 bytes |
| `5` | `miINT32` | 32-bit signed integer | 4 bytes |
| `6` | `miUINT32` | 32-bit unsigned integer | 4 bytes |
| `7` | `miSINGLE` | IEEE-754 32-bit float | 4 bytes |
| `9` | `miDOUBLE` | IEEE-754 64-bit float | 8 bytes |
| `12` | `miINT64` | 64-bit signed integer | 8 bytes |
| `13` | `miUINT64` | 64-bit unsigned integer | 8 bytes |
| `14` | `miMATRIX` | MATLAB Array container | Composite |
| `15` | `miCOMPRESSED` | Zlib deflate stream | Stream |
| `16` | `miUTF8` | UTF-8 encoded text | 1 byte |
| `17` | `miUTF16` | UTF-16 encoded text | 2 bytes |

---

## 5. Compressed Elements (`miCOMPRESSED = 15`)

In MATLAB v7 files, each top-level variable is encapsulated inside a `miCOMPRESSED` data element:
- Tag: Type = `15`, Size = length of compressed byte stream.
- Payload: Standard RFC 1950 zlib deflate compressed stream.
- **Decompression**: Passing the payload through `flate2::read::ZlibDecoder` expands into a single uncompressed `miMATRIX` element.

---

## 6. Matrix Data Element (`miMATRIX = 14`)

A `miMATRIX` payload is a compound element composed of ordered sub-elements:

```
+---------------------------------------------------------+
|                  miMATRIX Data Element                  |
|                                                         |
| 1. Array Flags (Tag: miUINT32, 8 bytes payload)         |
|    - Byte 0: Array Class (e.g. mxDOUBLE_CLASS = 6)      |
|    - Byte 1: Flags (Bit 1: Logical, Bit 2: Global,      |
|                     Bit 3: Complex)                     |
|                                                         |
| 2. Dimensions Array (Tag: miINT32, N*4 bytes payload)   |
|    - e.g. [M, N]                                        |
|                                                         |
| 3. Array Name (Tag: miINT8, length of name string)      |
|    - e.g. "my_variable\0"                               |
|                                                         |
| 4. Real Part (pr) (Tag: e.g. miDOUBLE, M*N*8 bytes)     |
|    - Column-major array of real numbers                 |
|                                                         |
| 5. Imaginary Part (pi) [OPTIONAL: if Complex flag set]  |
|    - Column-major array of imaginary numbers            |
+---------------------------------------------------------+
```

### 6.1 Array Classes (`mxClassID`)
- `0`: `mxUNKNOWN_CLASS`
- `1`: `mxCELL_CLASS`
- `2`: `mxSTRUCT_CLASS`
- `3`: `mxLOGICAL_CLASS`
- `4`: `mxCHAR_CLASS`
- `6`: `mxDOUBLE_CLASS`
- `7`: `mxSINGLE_CLASS`
- `8`: `mxINT8_CLASS`
- `9`: `mxUINT8_CLASS`
- `10`: `mxINT16_CLASS`
- `11`: `mxUINT16_CLASS`
- `12`: `mxINT32_CLASS`
- `13`: `mxUINT32_CLASS`
- `14`: `mxINT64_CLASS`
- `15`: `mxUINT64_CLASS`

### 6.2 Cell Arrays (`mxCELL_CLASS`)
Following the Array Name sub-element, a cell array contains $M \times N$ consecutive `miMATRIX` sub-elements, representing each cell entry in column-major order.

### 6.3 Struct Arrays (`mxSTRUCT_CLASS`)
Following the Array Name:
1. **Field Name Length Sub-element**: `miINT32` (typically 32 bytes).
2. **Field Names Sub-element**: `miINT8` containing names array (padded with zeros).
3. **Values**: Field values stored as consecutive `miMATRIX` elements ordered by (Field 1 for all elements, Field 2 for all elements, etc.).
