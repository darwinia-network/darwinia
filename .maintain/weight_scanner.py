#!/usr/bin/env python3
"""
Weight Scanner - Scan Rust weight files for maximum ref time and proof size

Usage:
    python weight_scanner.py <directory> [x]
"""

import os
import re
import argparse
from pathlib import Path
from typing import List, Tuple, Dict, Optional
from dataclasses import dataclass
import logging

@dataclass
class WeightResult:
    """Weight result data class."""
    file_path: str
    line_number: int
    function_name: str
    ref_time: int
    proof_size: int
    raw_expression: str

class WeightParser:
    """Weight parser that handles multiline expressions."""

    def __init__(self):
        # Match Weight::from_parts(ref_time, proof_size) with multiline support
        self.weight_pattern = re.compile(
            r'Weight::from_parts\s*\(\s*([^,]+?)\s*,\s*([^)]+?)\s*\)',
            re.MULTILINE | re.DOTALL
        )

        # Match function definitions
        self.function_pattern = re.compile(
            r'fn\s+(\w+)\s*\([^)]*\)\s*->\s*Weight\s*\{',
            re.MULTILINE | re.DOTALL
        )

        # Match saturating_add and saturating_mul with multiline support
        self.saturating_add_pattern = re.compile(
            r'\.saturating_add\s*\(\s*Weight::from_parts\s*\(\s*([^,]+?)\s*,\s*([^)]+?)\s*\)(?:\s*\.saturating_mul\s*\([^)]+?\))?\s*\)',
            re.MULTILINE | re.DOTALL
        )

        # Match complete weight expressions (including chained operations)
        self.complete_weight_pattern = re.compile(
            r'Weight::from_parts\s*\([^)]+\)(?:\s*\.(?:saturating_add|saturating_mul)\s*\([^)]+\))*',
            re.MULTILINE | re.DOTALL
        )

    def extract_numeric_value(self, expression: str) -> int:
        """Extract numeric value from expression, handling underscores and basic arithmetic."""
        try:
            # Remove whitespace and newlines
            expr = re.sub(r'\s+', '', expression.strip())

            # Handle Rust number separators (underscores)
            expr = re.sub(r'(\d)_(\d)', r'\1\2', expr)

            # Try direct integer conversion
            if expr.isdigit():
                return int(expr)

            # Handle basic arithmetic expressions (multiplication and addition only)
            # Example: "1000000", "2000000", "3652986"
            if re.match(r'^\d+$', expr):
                return int(expr)

            # If contains variables or complex expressions, return 0
            # These cases need more complex parsing
            return 0

        except (ValueError, AttributeError):
            return 0

    def parse_function_weights(self, content: str, file_path: str) -> List[WeightResult]:
        """Parse all function weights in the file."""
        results = []
        lines = content.split('\n')

        # Find all functions
        functions = []
        for match in self.function_pattern.finditer(content):
            func_name = match.group(1)
            func_start = content[:match.start()].count('\n') + 1
            functions.append((func_name, match.start(), func_start))

        # Analyze weights for each function
        for i, (func_name, func_start_pos, func_line) in enumerate(functions):
            # Determine function end position
            func_end_pos = len(content)
            if i + 1 < len(functions):
                func_end_pos = functions[i + 1][1]

            func_content = content[func_start_pos:func_end_pos]

            # Calculate total weight for the function
            total_ref_time = 0
            total_proof_size = 0
            weight_expressions = []

            # Find base weights
            for weight_match in self.weight_pattern.finditer(func_content):
                ref_time_expr = weight_match.group(1).strip()
                proof_size_expr = weight_match.group(2).strip()

                ref_time_val = self.extract_numeric_value(ref_time_expr)
                proof_size_val = self.extract_numeric_value(proof_size_expr)

                total_ref_time += ref_time_val
                total_proof_size += proof_size_val

                # Clean up expressions for display
                clean_ref_time = re.sub(r'\s+', ' ', ref_time_expr)
                clean_proof_size = re.sub(r'\s+', ' ', proof_size_expr)
                weight_expressions.append(f"Weight::from_parts({clean_ref_time}, {clean_proof_size})")

            # Find saturating_add weights
            for sat_match in self.saturating_add_pattern.finditer(func_content):
                ref_time_expr = sat_match.group(1).strip()
                proof_size_expr = sat_match.group(2).strip()

                ref_time_val = self.extract_numeric_value(ref_time_expr)
                proof_size_val = self.extract_numeric_value(proof_size_expr)

                # For saturating_add, we assume this is additional weight
                # In real scenarios, might need more complex analysis to handle .saturating_mul
                total_ref_time += ref_time_val
                total_proof_size += proof_size_val

                # Clean up expressions for display
                clean_ref_time = re.sub(r'\s+', ' ', ref_time_expr)
                clean_proof_size = re.sub(r'\s+', ' ', proof_size_expr)
                weight_expressions.append(f"saturating_add(Weight::from_parts({clean_ref_time}, {clean_proof_size}))")

            if total_ref_time > 0 or total_proof_size > 0:
                result = WeightResult(
                    file_path=file_path,
                    line_number=func_line,
                    function_name=func_name,
                    ref_time=total_ref_time,
                    proof_size=total_proof_size,
                    raw_expression=" + ".join(weight_expressions)
                )
                results.append(result)

        return results

    def scan_file(self, file_path: Path) -> List[WeightResult]:
        """扫描单个文件"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()

            return self.parse_function_weights(content, str(file_path))

        except Exception as e:
            logging.warning(f"Failed to parse {file_path}: {e}")
            return []

    def scan_directory(self, path: Path) -> List[WeightResult]:
        """Scan all Rust files in the directory or scan a single file."""
        all_results = []

        if path.is_file() and path.suffix == ".rs":
            # Handle single file
            results = self.scan_file(path)
            all_results.extend(results)
        elif path.is_dir():
            # Handle directory
            for rust_file in path.rglob("*.rs"):
                if rust_file.is_file():
                    results = self.scan_file(rust_file)
                    all_results.extend(results)

        return all_results

def format_file_link(file_path: str, line_number: int) -> str:
    """Format file link for VS Code click-to-jump functionality."""
    return f"{file_path}#L{line_number}"

def main():
    parser = argparse.ArgumentParser(
        description="Scan Rust weight files for maximum ref time and proof size"
    )
    parser.add_argument(
        "directory",
        type=Path,
        help="Directory path to scan or single .rs file"
    )
    parser.add_argument(
        "x",
        type=int,
        nargs="?",
        default=5,
        help="Number of top results to show (default: 5)"
    )

    args = parser.parse_args()

    if not args.directory.exists():
        print(f"Error: Path '{args.directory}' does not exist")
        return 1

    print(f"Scanning path: {args.directory}")
    print(f"Showing top {args.x} results")
    print("-" * 60)

    parser_instance = WeightParser()
    results = parser_instance.scan_directory(args.directory)

    if not results:
        print("No weight data found")
        return 0

    print(f"Total found {len(results)} weight functions")
    print()

    print("🔥 Maximum Ref Time Ranking:")
    print("-" * 60)
    sorted_by_ref_time = sorted(results, key=lambda x: x.ref_time, reverse=True)

    for i, result in enumerate(sorted_by_ref_time[:args.x], 1):
        file_link = format_file_link(result.file_path, result.line_number)
        print(f"{i:2d}. {result.ref_time:>15,} | {result.function_name}")
        print(f"     📁 {file_link}")
        print()

    print("📊 Maximum Proof Size Ranking:")
    print("-" * 60)
    sorted_by_proof_size = sorted(results, key=lambda x: x.proof_size, reverse=True)

    for i, result in enumerate(sorted_by_proof_size[:args.x], 1):
        file_link = format_file_link(result.file_path, result.line_number)
        print(f"{i:2d}. {result.proof_size:>15,} | {result.function_name}")
        print(f"     📁 {file_link}")
        print()

    return 0

if __name__ == "__main__":
    exit(main())
