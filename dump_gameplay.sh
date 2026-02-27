#!/bin/bash

TARGET_DIR="./src/screens/gameplay"
OUTPUT_FILE="gameplay.txt"

if [ ! -d "$TARGET_DIR" ]; then
  echo "La carpeta $TARGET_DIR no existe."
  exit 1
fi

echo "========== DUMP GAMEPLAY RS FILES ==========" > "$OUTPUT_FILE"
echo >> "$OUTPUT_FILE"

find "$TARGET_DIR" -type f -name "*.rs" | sort | while read file; do
  echo "============================================" >> "$OUTPUT_FILE"
  echo "FILE: $file" >> "$OUTPUT_FILE"
  echo "============================================" >> "$OUTPUT_FILE"
  nl -ba "$file" >> "$OUTPUT_FILE"
  echo >> "$OUTPUT_FILE"
  echo >> "$OUTPUT_FILE"
done

echo "============== END OF DUMP =================" >> "$OUTPUT_FILE"

echo "Dump completado en $OUTPUT_FILE"
