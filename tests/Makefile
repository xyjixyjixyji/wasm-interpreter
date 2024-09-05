# Directories
CTESTS_DIR = ctests
WATTSTS_DIR = wattests
OUTPUT_DIR = wasm

# Tools
CLANG = clang
WAT2WASM = wat2wasm
WASM2WAT = wasm2wat

# File extensions
C_EXT = .c
WAT_EXT = .wat
WASM_EXT = .wasm

# Find all C files and WAT files
C_FILES = $(wildcard $(CTESTS_DIR)/*$(C_EXT))
WAT_FILES = $(wildcard $(WATTSTS_DIR)/*$(WAT_EXT))

# Define WASM files from C and WAT sources
WASM_FILES_FROM_C = $(patsubst $(CTESTS_DIR)/%$(C_EXT),$(OUTPUT_DIR)/%$(WASM_EXT),$(C_FILES))
WASM_FILES_FROM_WAT = $(patsubst $(WATTSTS_DIR)/%$(WAT_EXT),$(OUTPUT_DIR)/%$(WASM_EXT),$(WAT_FILES))

# All targets
all: $(WASM_FILES_FROM_C) $(WASM_FILES_FROM_WAT)

# Rule to build WASM from C files
$(OUTPUT_DIR)/%$(WASM_EXT): $(CTESTS_DIR)/%$(C_EXT)
	@mkdir -p $(OUTPUT_DIR)
	$(CLANG) --target=wasm32 -O0 -nostartfiles -nostdlib -Wl,--no-entry -o $@ $<

# Rule to convert WAT files to WASM
$(OUTPUT_DIR)/%$(WASM_EXT): $(WATTSTS_DIR)/%$(WAT_EXT)
	@mkdir -p $(OUTPUT_DIR)
	$(WAT2WASM) $< -o $@

# Clean up generated files
clean:
	rm -rf $(OUTPUT_DIR)

.PHONY: all clean

