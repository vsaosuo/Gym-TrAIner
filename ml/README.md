## Installation
```bash
pip install -r requirements.txt
```
Make sure you're in directory with requiremente.txt file

## Run Program
Pass video file directory as the first parameter
```bash
python squatPredictor.py VIDEO_FILE_DIR DST_VIDEO_NAME
```

<br>Output: print out result on terminal in JSON string

## Libraries
```bash
matplotlib==3.6.2
mediapipe==0.9.1.0
numpy==1.22.3
opencv_contrib_python==4.7.0.72
pandas==1.5.2
scikit_learn==1.2.1
tensorflow==2.11.0

```
For Mac users 
```bash
mediapipe_silicon==0.9.1
tensorflow_macos==2.11.0
```

### Main file used:
- squatPredictor.py
- pushupPredictor.py
- lib/utils.py
- squatModel.h5
- squatModel_lessClass.h5
- squatML_trainer.ipynb
