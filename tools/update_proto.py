#!/usr/bin/env python3
"""
Update liqi_config and regenerate proto/liqi.proto and proto/sheets.proto.

This script does two main things:
  1. Fetch the latest `liqi.json` and `lqc.lqbin` from game.maj-soul.com and write
     them into ./liqi_config.
  2. Regenerate:
       - ./proto/liqi.proto from ./liqi_config/liqi.json (protobufjs JSON, package lq)
       - ./proto/sheets.proto from ./liqi_config/lqc.lqbin (lq.config.ConfigTables)
     so that the Rust code uses up‑to‑date protocol definitions.

Notes:
  - You need the `requests` Python package installed:
        pip install requests
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Any, List, Tuple

import requests


ROOT = Path(__file__).resolve().parents[1]
LIQI_CONFIG_DIR = ROOT / "liqi_config"
PROTO_DIR = ROOT / "proto"

GAME_BASE = "https://game.maj-soul.com/1"


def fetch_version() -> Dict[str, Any]:
    resp = requests.get(f"{GAME_BASE}/version.json", timeout=10)
    resp.raise_for_status()
    return resp.json()


def fetch_resversion(version: str) -> Dict[str, Any]:
    resp = requests.get(f"{GAME_BASE}/resversion{version}.json", timeout=10)
    resp.raise_for_status()
    return resp.json()


def update_liqi_config() -> None:
    """
    Download latest liqi.json and lqc.lqbin into ./liqi_config.
    """
    LIQI_CONFIG_DIR.mkdir(exist_ok=True)

    version_info = fetch_version()
    version = version_info["version"]
    res = fetch_resversion(version)

    liqi_prefix = (
        res.get("res", {}).get("res/proto/liqi.json", {}).get("prefix", "")
    )
    lqc_prefix = (
        res.get("res", {}).get("res/config/lqc.lqbin", {}).get("prefix", "")
    )

    if not liqi_prefix or not lqc_prefix:
        raise RuntimeError("Failed to get liqi/lqc prefixes from resversion.json")

    print(f"[liqi_config] game version={version}, liqi_prefix={liqi_prefix}, "
          f"lqc_prefix={lqc_prefix}")

    # Download liqi.json
    liqi_url = f"{GAME_BASE}/{liqi_prefix}/res/proto/liqi.json"
    print(f"[liqi_config] downloading {liqi_url}")
    liqi_resp = requests.get(liqi_url, timeout=20)
    liqi_resp.raise_for_status()
    (LIQI_CONFIG_DIR / "liqi.json").write_text(liqi_resp.text, encoding="utf-8")

    # Download lqc.lqbin
    lqc_url = f"{GAME_BASE}/{lqc_prefix}/res/config/lqc.lqbin"
    print(f"[liqi_config] downloading {lqc_url}")
    lqc_resp = requests.get(lqc_url, timeout=20)
    lqc_resp.raise_for_status()
    (LIQI_CONFIG_DIR / "lqc.lqbin").write_bytes(lqc_resp.content)

    # Best‑effort: update version fields in settings files if present.
    for settings_name, key in (
        ("settings.json", "liqiVersion"),
        ("settings.mod.json", "version"),
    ):
        path = LIQI_CONFIG_DIR / settings_name
        if not path.exists():
            continue
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            continue
        if data.get(key) != liqi_prefix:
            data[key] = liqi_prefix
            path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
            print(f"[liqi_config] updated {settings_name}:{key} -> {liqi_prefix}")


# ===== liqi.json (protobufjs JSON) -> liqi.proto =====

def escape_str(value: str) -> str:
    return value.replace("\\", "\\\\").replace('"', '\\"')


def emit_enum(name: str, desc: Dict[str, Any], indent: str, out: List[str]) -> None:
    out.append(f"{indent}enum {name} {{\n")
    values: Dict[str, int] = desc.get("values", {})
    # Sort by numeric value for stability
    for v_name, v_num in sorted(values.items(), key=lambda kv: kv[1]):
        out.append(f"{indent}    {v_name} = {v_num};\n")
    out.append(f"{indent}}}\n\n")


def _field_type_and_label(field: Dict[str, Any]) -> Tuple[str, str]:
    """
    Return (type_str, label_str) where label_str is usually "" or "repeated ".
    Map fields are rendered as map<key, value> with empty label.
    """
    # Map field
    if "keyType" in field:
        key_t = field["keyType"]
        val_t = field["type"]
        return f"map<{key_t}, {val_t}>", ""

    label = ""
    if field.get("rule") == "repeated":
        label = "repeated "
    return field["type"], label


def emit_message(name: str, desc: Dict[str, Any], indent: str, out: List[str]) -> None:
    out.append(f"{indent}message {name} {{\n")

    fields: Dict[str, Dict[str, Any]] = desc.get("fields", {})
    # Map field name -> oneof group name
    oneofs: Dict[str, str] = {}
    for oneof_name, field_names in desc.get("oneofs", {}).items():
        for fname in field_names:
            oneofs[fname] = oneof_name

    # Emit oneof groups first
    handled: set[str] = set()
    # Preserve oneof declaration order by iterating over desc["oneofs"]
    for oneof_name, field_names in desc.get("oneofs", {}).items():
        out.append(f"{indent}    oneof {oneof_name} {{\n")
        for fname in field_names:
            field = fields[fname]
            f_type, _ = _field_type_and_label(field)
            out.append(
                f"{indent}        {f_type} {fname} = {field['id']};\n"
            )
            handled.add(fname)
        out.append(f"{indent}    }}\n")

    # Emit remaining fields ordered by tag id
    ordered_fields = sorted(fields.items(), key=lambda kv: kv[1]["id"])
    for fname, field in ordered_fields:
        if fname in handled:
            continue
        f_type, label = _field_type_and_label(field)
        out.append(
            f"{indent}    {label}{f_type} {fname} = {field['id']};\n"
        )

    # Nested types inside the message
    nested = desc.get("nested", {})
    if nested:
        out.append("\n")
        emit_nested(nested, indent + "    ", out)

    out.append(f"{indent}}}\n\n")


def emit_service(name: str, desc: Dict[str, Any], indent: str, out: List[str]) -> None:
    out.append(f"{indent}service {name} {{\n")
    methods: Dict[str, Dict[str, Any]] = desc.get("methods", {})
    for m_name, m in methods.items():
        req = m.get("requestType", "google.protobuf.Empty")
        resp = m.get("responseType", "google.protobuf.Empty")
        out.append(
            f"{indent}    rpc {m_name}({req}) returns ({resp});\n"
        )
    out.append(f"{indent}}}\n\n")


def emit_nested(nested: Dict[str, Any], indent: str, out: List[str]) -> None:
    # Keep deterministic order
    for name in sorted(nested.keys()):
        desc = nested[name]
        if "fields" in desc or "oneofs" in desc:
            emit_message(name, desc, indent, out)
        elif "values" in desc:
            emit_enum(name, desc, indent, out)
        elif "methods" in desc:
            emit_service(name, desc, indent, out)
        else:
            # A bare namespace with only nested types
            inner = desc.get("nested")
            if inner:
                emit_nested(inner, indent, out)


def regenerate_liqi_proto() -> None:
    """
    Read liqi_config/liqi.json (protobufjs JSON) and regenerate proto/liqi.proto.
    """
    liqi_json_path = LIQI_CONFIG_DIR / "liqi.json"
    if not liqi_json_path.exists():
        raise FileNotFoundError(f"{liqi_json_path} not found; run with --download first")

    data = json.loads(liqi_json_path.read_text(encoding="utf-8"))
    root_nested = data.get("nested") or {}
    if "lq" not in root_nested:
        raise RuntimeError("liqi.json does not contain top‑level package 'lq'")

    lq_desc = root_nested["lq"]

    out: List[str] = []
    out.append('syntax = "proto3";\n\n')
    out.append("package lq;\n\n")

    # Preserve go_package option if present
    go_pkg = (lq_desc.get("options") or {}).get("go_package")
    if go_pkg:
        out.append(f'option go_package = "{escape_str(go_pkg)}";\n\n')

    emit_nested(lq_desc.get("nested", {}), "", out)

    proto_text = "".join(out)
    PROTO_DIR.mkdir(exist_ok=True)
    target = PROTO_DIR / "liqi.proto"
    target.write_text(proto_text, encoding="utf-8")
    print(f"[proto] regenerated {target}")


# ===== lqc.lqbin (ConfigTables) -> sheets.proto =====


@dataclass
class FieldDesc:
    field_name: str
    array_length: int
    pb_type: str
    pb_index: int


@dataclass
class SheetDesc:
    table_name: str
    sheet_name: str
    fields: List[FieldDesc]


@dataclass
class TableSchemaDesc:
    name: str
    sheets: List[SheetDesc]


class _ProtoReader:
    """
    Minimal protobuf wire-format reader for the small subset used by
    lq.config.ConfigTables (varints + length-delimited strings/messages).
    """

    def __init__(self, data: bytes):
        self._data = data
        self._pos = 0
        self._len = len(data)

    def eof(self) -> bool:
        return self._pos >= self._len

    def _read_raw_byte(self) -> int:
        if self._pos >= self._len:
            raise EOFError("Unexpected EOF while reading varint")
        b = self._data[self._pos]
        self._pos += 1
        return b

    def read_varint(self) -> int:
        shift = 0
        result = 0
        while True:
            b = self._read_raw_byte()
            result |= (b & 0x7F) << shift
            if (b & 0x80) == 0:
                break
            shift += 7
            if shift >= 64:
                raise ValueError("Varint too long")
        return result

    def read_length_delimited(self) -> bytes:
        length = self.read_varint()
        end = self._pos + length
        if end > self._len:
            raise EOFError("Unexpected EOF while reading length-delimited field")
        chunk = self._data[self._pos:end]
        self._pos = end
        return chunk

    def skip_field(self, wire_type: int) -> None:
        if wire_type == 0:  # varint
            _ = self.read_varint()
        elif wire_type == 1:  # 64-bit
            self._pos += 8
        elif wire_type == 2:  # length-delimited
            length = self.read_varint()
            self._pos += length
        elif wire_type == 5:  # 32-bit
            self._pos += 4
        else:
            raise ValueError(f"Unsupported wire type: {wire_type}")


def _parse_field_desc(data: bytes) -> FieldDesc:
    r = _ProtoReader(data)
    field_name = ""
    array_length = 0
    pb_type = ""
    pb_index = 0

    while not r.eof():
        key = r.read_varint()
        field_number = key >> 3
        wire_type = key & 0x7

        if field_number == 1 and wire_type == 2:
            field_name = r.read_length_delimited().decode("utf-8")
        elif field_number == 2 and wire_type == 0:
            array_length = r.read_varint()
        elif field_number == 3 and wire_type == 2:
            pb_type = r.read_length_delimited().decode("utf-8")
        elif field_number == 4 and wire_type == 0:
            pb_index = r.read_varint()
        else:
            r.skip_field(wire_type)

    return FieldDesc(
        field_name=field_name,
        array_length=array_length,
        pb_type=pb_type,
        pb_index=pb_index,
    )


def _parse_sheet_desc(table_name: str, data: bytes) -> SheetDesc:
    r = _ProtoReader(data)
    sheet_name = ""
    fields: List[FieldDesc] = []

    while not r.eof():
        key = r.read_varint()
        field_number = key >> 3
        wire_type = key & 0x7

        if field_number == 1 and wire_type == 2:
            sheet_name = r.read_length_delimited().decode("utf-8")
        elif field_number == 3 and wire_type == 2:
            field_bytes = r.read_length_delimited()
            fields.append(_parse_field_desc(field_bytes))
        else:
            # We don't need meta (field 2) or other fields for proto generation.
            r.skip_field(wire_type)

    return SheetDesc(table_name=table_name, sheet_name=sheet_name, fields=fields)


def _parse_table_schema_desc(data: bytes) -> TableSchemaDesc:
    r = _ProtoReader(data)
    name = ""
    sheets: List[SheetDesc] = []

    while not r.eof():
        key = r.read_varint()
        field_number = key >> 3
        wire_type = key & 0x7

        if field_number == 1 and wire_type == 2:
            name = r.read_length_delimited().decode("utf-8")
        elif field_number == 2 and wire_type == 2:
            sheet_bytes = r.read_length_delimited()
            sheets.append(_parse_sheet_desc(name, sheet_bytes))
        else:
            r.skip_field(wire_type)

    return TableSchemaDesc(name=name, sheets=sheets)


def _parse_config_tables_schemas(data: bytes) -> List[TableSchemaDesc]:
    """
    Parse lq.config.ConfigTables and return only its schemas (table definitions).
    We ignore the SheetData (datas field) entirely.
    """
    r = _ProtoReader(data)
    tables: List[TableSchemaDesc] = []

    while not r.eof():
        key = r.read_varint()
        field_number = key >> 3
        wire_type = key & 0x7

        if field_number == 3 and wire_type == 2:
            schema_bytes = r.read_length_delimited()
            tables.append(_parse_table_schema_desc(schema_bytes))
        else:
            # version (1), header_hash (2), datas (4), and any unknowns are skipped.
            r.skip_field(wire_type)

    return tables


def _camelize(name: str) -> str:
    # Match the Rust capitalize() logic: uppercase first char, keep rest as-is.
    parts = (p for p in name.split("_") if p)
    return "".join(p[:1].upper() + p[1:] for p in parts)


def _map_pb_type(pb_type: str) -> str:
    # pb_type only uses these four in lqc.lqbin; keep a guard in case it changes.
    if pb_type in {"uint32", "int32", "float", "string"}:
        return pb_type
    raise ValueError(f"Unsupported pb_type in lqc.lqbin: {pb_type!r}")


def regenerate_sheets_proto() -> None:
    """
    Read liqi_config/lqc.lqbin (lq.config.ConfigTables) and regenerate proto/sheets.proto.
    """
    lqc_path = LIQI_CONFIG_DIR / "lqc.lqbin"
    if not lqc_path.exists():
        raise FileNotFoundError(
            f"{lqc_path} not found; run without --no-download first to fetch it"
        )

    schemas = _parse_config_tables_schemas(lqc_path.read_bytes())

    out: List[str] = []
    out.append('syntax = "proto3";\n\n')
    out.append("package sheets;\n\n")

    for table in schemas:
        table_prefix = _camelize(table.name)
        for sheet in table.sheets:
            msg_name = table_prefix + _camelize(sheet.sheet_name)
            out.append(f"message {msg_name} {{\n")

            # Sort by pb_index for deterministic output, just like proto fields.
            for field in sorted(sheet.fields, key=lambda f: f.pb_index):
                if not field.field_name or not field.pb_type or field.pb_index == 0:
                    # Skip incomplete field defs defensively.
                    continue
                label = "repeated " if field.array_length else ""
                type_str = _map_pb_type(field.pb_type)
                out.append(
                    f"   {label}{type_str} {field.field_name} = {field.pb_index};\n"
                )

            out.append("}\n\n")

    proto_text = "".join(out)
    PROTO_DIR.mkdir(exist_ok=True)
    target = PROTO_DIR / "sheets.proto"
    target.write_text(proto_text, encoding="utf-8")
    print(f"[proto] regenerated {target}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Update liqi_config and regenerate proto/liqi.proto and proto/sheets.proto from official Mahjong Soul endpoints.",
    )
    parser.add_argument(
        "--no-download",
        action="store_true",
        help="Skip downloading from the official servers and only regenerate proto/liqi.proto "
        "from existing liqi_config/liqi.json.",
    )
    args = parser.parse_args()

    if not args.no_download:
        update_liqi_config()
    regenerate_liqi_proto()
    regenerate_sheets_proto()


if __name__ == "__main__":
    main()