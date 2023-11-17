import numpy as np
from sklearn.preprocessing import MinMaxScaler
from tensorflow.keras.models import load_model
import cv2
import mediapipe as mp
import json
import sys
import lib.utils as utils

arguments = sys.argv[1:]

videoFile = arguments[0]
destDir = arguments[1]
derive_features = utils.derive_features
calculate_angle = utils.calculate_angle
calculate_distance = utils.calculate_distance
draw_XY_angle = utils.draw_XY_angle
mp_drawing = mp.solutions.drawing_utils
draw_Count = utils.draw_Count
mp_pose = mp.solutions.pose


####### Pose Detection #######
# Make Detection
cap = cv2.VideoCapture(videoFile)
squat_count = 0
state_count = 0
S_angle = 130
confident_threshold = 0.5
smallestKneeAngle = S_angle
# Store an array of array of features for squatRec[i][j] represent ith squat and jth keyth points of the smallest knee angle frame
exerciseRec = [[]]
i = 0

width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))

outVideo = cv2.VideoWriter(destDir, cv2.VideoWriter_fourcc(
    *'avc1'), 30, (width, height), True)

# outVideo = cv2.VideoWriter(destDir + "/outvideo.mp4", cv2.VideoWriter_fourcc('M','J','P','G'), fps, (frame_width,frame_height))


# Function block below is derived from Nicholas Renotte tutorials
# Setup mediapipe instance
with mp_pose.Pose(min_detection_confidence=confident_threshold, min_tracking_confidence=confident_threshold) as pose:
    while cap.isOpened():
        ret, frame = cap.read()

        # Recolor image to RGB
        try:
            image = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            image.flags.writeable = False
        except:
            break

        # Make detection
        results = pose.process(image)

        # Recolor back to BGR
        image.flags.writeable = True
        image = cv2.cvtColor(image, cv2.COLOR_RGB2BGR)

        # Render detections
        mp_drawing.draw_landmarks(image, results.pose_landmarks, mp_pose.POSE_CONNECTIONS,
                                  mp_drawing.DrawingSpec(
                                      color=(245, 117, 66), thickness=2, circle_radius=2),
                                  mp_drawing.DrawingSpec(color=(245, 66, 230), thickness=2, circle_radius=2))

        # Extract landmarks
        try:
            lm = results.pose_landmarks.landmark
            pl = mp_pose.PoseLandmark

            # Compute angle
            ar = [lm[pl.RIGHT_HIP.value].x, lm[pl.RIGHT_HIP.value].y]
            br = [lm[pl.RIGHT_KNEE.value].x, lm[pl.RIGHT_KNEE.value].y]
            cr = [lm[pl.RIGHT_ANKLE.value].x, lm[pl.RIGHT_ANKLE.value].y]

            al = [lm[pl.LEFT_HIP.value].x, lm[pl.LEFT_HIP.value].y]
            bl = [lm[pl.LEFT_KNEE.value].x, lm[pl.LEFT_KNEE.value].y]
            cl = [lm[pl.LEFT_ANKLE.value].x, lm[pl.LEFT_ANKLE.value].y]

            angleR = calculate_angle(ar, br, cr)
            angleL = calculate_angle(al, bl, cl)

            draw_Count(image, "", squat_count)

            # Frame Selection
            minKnee = min(angleR, angleL)

            if (minKnee < S_angle):  # down position
                if (state_count == 0):
                    state_count += 1

                if (minKnee < smallestKneeAngle):
                    smallestKneeAngle = minKnee
                    features_i = [lm[pl.LEFT_SHOULDER.value], lm[pl.RIGHT_SHOULDER.value], lm[pl.LEFT_HIP.value], lm[pl.RIGHT_HIP.value], lm[pl.LEFT_KNEE.value], lm[pl.RIGHT_KNEE.value],
                                  lm[pl.LEFT_ANKLE.value], lm[pl.RIGHT_ANKLE.value], lm[pl.LEFT_HEEL.value], lm[pl.RIGHT_HEEL.value], lm[pl.LEFT_FOOT_INDEX.value], lm[pl.RIGHT_FOOT_INDEX.value]]
                    exerciseRec[i] = features_i

            elif (minKnee > S_angle):  # stand position
                if (state_count == 1):   # Already down position
                    state_count = 0
                    squat_count += 1
                    i += 1
                    smallestKneeAngle = minKnee
                    exerciseRec.append([])

        except:
            pass

        # Write into frame
        outVideo.write(image)

# Output video
outVideo.release()

####### Data Processor #######
inputPreData = []
for exer_i_th in range(len(exerciseRec) - 1):  # last exercise is just uncounted
    df = derive_features(exerciseRec[exer_i_th])
    inputPreData.append(df)

####### Pose Classification #######
model = load_model('.ml/squatModel_lessClass.h5')
feedback = []
squatClass = [
    {'name': 'Acceptable', 'description': 'Normal squat'},
    {'name': 'Anterior Knee', 'description': 'Knee ahead of toes during exercise'},
    {'name': 'Bent Over', 'description': 'Excessive flexing of hip and torso'},
    {'name': 'Knee Valgus', 'description': 'Both knees pointing inside during exercise'},
    {'name': 'Knee Varus', 'description': 'Both knees pointing outside during exercise'},
    {'name': 'Half Squat', 'description': 'Insufficient squatting depth'},
    {'name': 'Other', 'description': 'Probably standing most of the time'}
]

# Using https://www.raynersmale.com/blog/2014/1/31/optimising-your-squat
# left_knee_angle, left_hip_angle, left_ankle_angle, right_knee_angle, right_hip_angle, right_ankle_angle
# Data is obtained from assets/_squat_perfect.mp4
perfectSquat = [84.87369318857579, 83.45166654986053, 153.37894731195783,
                85.30513286618259, 83.7632826648259, 179.36188848969115]

acceptedMultiplier = 0
acceptedAngleTolerance = 7
devisorTaken = 1.5
for exer_i_th in range(len(inputPreData)):
    # Preliminary check for Acceptability
    largest = 0

    # Take percenter diference form the perfect form
    percentSumDiffer = 0
    for i in range(6):
        diff = abs(perfectSquat[i] - inputPreData[exer_i_th][i])

        if (diff <= acceptedAngleTolerance):
            percentDiff = 1
        else:
            # Percent diference from tolerated angle
            percentDiff = 1 - (diff/(perfectSquat[i] - acceptedAngleTolerance))

        percentSumDiffer += percentDiff

    # Get average difference
    percentSumDiffer = percentSumDiffer/6

    # Convert to np array
    exer_i = np.array(inputPreData[exer_i_th])

    # Normalize the data to [0-1]
    scaler = MinMaxScaler(feature_range=(0, 1))
    # reshape since fit_transform does not take 1D array
    exer_i = scaler.fit_transform(exer_i.reshape(-1, 1))

    exer_i = exer_i.reshape(-1, 21)

    # Run prediction on the sample data
    predictions = model.predict(x=exer_i, batch_size=10, verbose=0)

    ######## Feedback Generator #######

    # Aggregate for Acceptable Class
    # Take only 1 half weight from each class
    percentSumDiffer = percentSumDiffer/devisorTaken
    for class_i in range(1, 5):
        weightTaken = percentSumDiffer*predictions[0][class_i]
        predictions[0][class_i] -= weightTaken

        predictions[0][0] += weightTaken

    # largest = predictions.argmax()
    # weightTaken = percentSumDiffer*predictions[0][largest]
    # predictions[0][largest] -= weightTaken
    # predictions[0][0] += weightTaken

    largest = predictions.argmax()

    fb_i = {
        'ex_number': exer_i_th + 1,
        'class': squatClass[largest]['name'],
        'correction': squatClass[largest]['description'],
        'confidence': predictions[0][largest] * 100
    }

    feedback.append(fb_i)

print(json.dumps(feedback))
