"""
Compare native-iqa BRISQUE scores against OpenCV's own cv2.quality.QualityBRISQUE_compute.
Both implementations use brisque_model_live.yml + brisque_range_live.yml (LIVE IQA database).
This is the authoritative reference — piq.brisque uses a different model.

Run:
    /opt/homebrew/anaconda3/envs/eicon/bin/python3 bench/compare_opencv.py
    (or: conda activate eicon && python bench/compare_opencv.py)
"""

import sys
import os
import csv
import subprocess
import numpy as np

try:
    import cv2
    assert hasattr(cv2, 'quality'), "opencv-contrib not installed"
except (ImportError, AssertionError) as e:
    sys.exit(f"Missing deps: {e}\n  conda activate eicon && pip install opencv-contrib-python")


BENCH_DIR   = os.path.dirname(__file__)
IMAGES_DIR  = os.path.join(BENCH_DIR, "images")
MODEL_FILE  = os.path.join(BENCH_DIR, "brisque_model_live.yml")
RANGE_FILE  = os.path.join(BENCH_DIR, "brisque_range_live.yml")
OUTPUT_CSV  = os.path.join(BENCH_DIR, "reference_scores.csv")

if not os.path.exists(MODEL_FILE):
    sys.exit(f"Model not found: {MODEL_FILE}\n  Run: curl -sL https://raw.githubusercontent.com/opencv/opencv_contrib/master/modules/quality/samples/brisque_model_live.yml -o bench/brisque_model_live.yml")

# --------------------------------------------------------------------------
# Generate test images (same as compare_reference.py but standalone)
# --------------------------------------------------------------------------

IMAGE_NAMES = [
    "astronaut_sharp.jpg",
    "astronaut_blur.jpg",
    "astronaut_noise.jpg",
    "astronaut_dark.jpg",
    "astronaut_bright.jpg",
    "camera_sharp.jpg",
    "camera_blur.jpg",
    "camera_noise.jpg",
]

missing = [n for n in IMAGE_NAMES if not os.path.exists(os.path.join(IMAGES_DIR, n))]
if missing:
    sys.exit(f"Missing images: {missing}\n  Run: python bench/compare_reference.py")

# --------------------------------------------------------------------------
# Score with OpenCV BRISQUE
# --------------------------------------------------------------------------

brisque = cv2.quality.QualityBRISQUE_create(MODEL_FILE, RANGE_FILE)

print(f"\n{'Image':<35} {'opencv':>8}")
print("-" * 45)

rows = []
for name in IMAGE_NAMES:
    path = os.path.join(IMAGES_DIR, name)
    img_bgr = cv2.imread(path)
    score = brisque.compute(img_bgr)[0]
    rows.append({"image": name, "reference_score": f"{score:.4f}"})
    print(f"  {name:<33} {score:>8.2f}")

with open(OUTPUT_CSV, "w", newline="") as f:
    writer = csv.DictWriter(f, fieldnames=["image", "reference_score"])
    writer.writeheader()
    writer.writerows(rows)

print(f"\nScores → {OUTPUT_CSV}")
print(f"Next:    node bench/compare_reference.js")
