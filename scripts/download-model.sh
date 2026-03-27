#!/bin/bash
# Download Whisper model for Voix
# Usage: ./scripts/download-model.sh [tiny|base|small|medium]

set -e

MODEL_SIZE=${1:-small}
MODEL_URL_BASE="https://huggingface.co/ggerganov/whisper.cpp/resolve/main"

mkdir -p models

case $MODEL_SIZE in
    tiny)
        FILE="ggml-tiny.en.bin"
        ;;
    base)
        FILE="ggml-base.en.bin"
        ;;
    small)
        FILE="ggml-small.en.bin"
        ;;
    medium)
        FILE="ggml-medium.bin"
        ;;
    *)
        echo "Unknown model: $MODEL_SIZE"
        echo "Valid options: tiny, base, small, medium"
        exit 1
        ;;
esac

echo "Downloading Whisper $MODEL_SIZE model..."
curl -L -o "models/$FILE" "$MODEL_URL_BASE/$FILE"
echo "Done. Model saved to models/$FILE"
