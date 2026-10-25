#!/usr/bin/env python3
"""
MAT-File (Level 5 / Version 7) Interoperability Test Suite Generator.
Generates a comprehensive suite of Level 5/v7 .mat files using scipy.io.savemat
covering all fundamental MATLAB data types, shapes, complex numbers, sparse
matrices, containers, and compression modes.
"""

import warnings
import os
import sys
import numpy as np
import scipy.io as sio
import scipy.sparse as sp

warnings.filterwarnings("ignore", category=DeprecationWarning)

def get_output_dir():
    # Fixtures live INSIDE the repo so tests never depend on `target/` state:
    # rustlab/crates/rustlab-matio/tests/data
    script_dir = os.path.dirname(os.path.abspath(__file__))
    rustlab_dir = os.path.dirname(script_dir)
    out_dir = os.path.join(rustlab_dir, "crates", "rustlab-matio", "tests", "data")
    os.makedirs(out_dir, exist_ok=True)
    return out_dir

def create_test_dataset():
    """Builds a dictionary containing representative data for all MATLAB v7 data types."""
    data = {}

    # 1. Double precision (real and complex)
    data["var_double_real"] = np.array([[1.0, 2.5, -3.14159], [0.0, 1e-10, 1e10]], dtype=np.float64)
    data["var_double_complex"] = np.array([[1.0 + 2.0j, -3.5 + 0.5j], [0.0 - 1.0j, 4.2 + 0.0j]], dtype=np.complex128)

    # 2. Single precision (float32)
    data["var_single"] = np.array([[1.5, -2.25], [3.125, 0.0]], dtype=np.float32)

    # 3. Signed and Unsigned Integers
    data["var_int8"] = np.array([[-128, 0], [64, 127]], dtype=np.int8)
    data["var_uint8"] = np.array([[0, 128], [200, 255]], dtype=np.uint8)
    data["var_int16"] = np.array([[-32768, 0], [16384, 32767]], dtype=np.int16)
    data["var_uint16"] = np.array([[0, 32768], [50000, 65535]], dtype=np.uint16)
    data["var_int32"] = np.array([[-2147483648, 0], [100000, 2147483647]], dtype=np.int32)
    data["var_uint32"] = np.array([[0, 1000000], [2000000000, 4294967295]], dtype=np.uint32)
    data["var_int64"] = np.array([[-9223372036854775808, 0], [1000, 9223372036854775807]], dtype=np.int64)
    data["var_uint64"] = np.array([[0, 1000], [1000000000, 18446744073709551615]], dtype=np.uint64)

    # 4. Character arrays / strings
    data["var_char"] = np.array("RustLab MAT-File Spec Validation")

    # 5. Logical / boolean
    data["var_logical"] = np.array([[True, False], [False, True]], dtype=np.bool_)

    # 6. Heterogeneous Cell array
    # In scipy.io, cell arrays are object arrays containing elements
    cell_arr = np.empty((2, 2), dtype=object)
    cell_arr[0, 0] = np.array([[1.0, 2.0], [3.0, 4.0]])
    cell_arr[0, 1] = "SubString"
    cell_arr[1, 0] = np.array([[10, 20]], dtype=np.int32)
    cell_arr[1, 1] = np.array([[True, False]])
    data["var_cell"] = cell_arr

    # 7. Struct array (multi-field)
    struct_val = {
        "field_name": "RustLabEngine",
        "field_id": 42.0,
        "field_matrix": np.array([[1.0, 0.0], [0.0, 1.0]], dtype=np.float64),
    }
    data["var_struct"] = struct_val

    # 8. Sparse matrix (CSC format)
    row = np.array([0, 2, 2, 0, 1, 2])
    col = np.array([0, 0, 1, 2, 2, 2])
    val = np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0], dtype=np.float64)
    data["var_sparse"] = sp.csc_matrix((val, (row, col)), shape=(3, 3))

    # 9. N-D Tensor (3D array: 2x3x4)
    data["var_tensor_3d"] = np.arange(24, dtype=np.float64).reshape((2, 3, 4), order="F")

    # 10. Empty matrices
    data["var_empty_0x0"] = np.empty((0, 0), dtype=np.float64)
    data["var_empty_0x5"] = np.empty((0, 5), dtype=np.float64)

    return data

def generate_and_validate():
    out_dir = get_output_dir()
    print(f"[+] Output directory: {out_dir}")

    data = create_test_dataset()

    # Generate individual files (both compressed and uncompressed)
    summary = []

    for name, value in data.items():
        dict_payload = {name: value}

        # Compressed variant (Level 5 / v7 default)
        comp_filename = f"{name}_compressed.mat"
        comp_path = os.path.join(out_dir, comp_filename)
        sio.savemat(comp_path, dict_payload, do_compression=True)

        # Uncompressed variant (Level 5)
        uncomp_filename = f"{name}_uncompressed.mat"
        uncomp_path = os.path.join(out_dir, uncomp_filename)
        sio.savemat(uncomp_path, dict_payload, do_compression=False)

        # Verify roundtrip with scipy.io.loadmat
        loaded_comp = sio.loadmat(comp_path)
        loaded_uncomp = sio.loadmat(uncomp_path)

        assert name in loaded_comp, f"Failed roundtrip read of compressed {comp_filename}"
        assert name in loaded_uncomp, f"Failed roundtrip read of uncompressed {uncomp_filename}"

        comp_size = os.path.getsize(comp_path)
        uncomp_size = os.path.getsize(uncomp_path)

        summary.append({
            "type_name": name,
            "compressed_file": comp_filename,
            "compressed_size": comp_size,
            "uncompressed_file": uncomp_filename,
            "uncompressed_size": uncomp_size,
        })
        print(f"  [OK] {name:<22} -> Comp: {comp_size:>5} bytes | Uncomp: {uncomp_size:>6} bytes")

    # Also generate comprehensive all-in-one files
    all_comp_path = os.path.join(out_dir, "suite_all_types_compressed.mat")
    sio.savemat(all_comp_path, data, do_compression=True)
    all_uncomp_path = os.path.join(out_dir, "suite_all_types_uncompressed.mat")
    sio.savemat(all_uncomp_path, data, do_compression=False)

    print(f"\n[+] Generated full suite: {all_comp_path} ({os.path.getsize(all_comp_path)} bytes)")
    print(f"[+] Generated full suite: {all_uncomp_path} ({os.path.getsize(all_uncomp_path)} bytes)")

    print(f"\n[SUCCESS] All {len(summary)} types successfully generated and verified with SciPy.")
    return summary

if __name__ == "__main__":
    generate_and_validate()
