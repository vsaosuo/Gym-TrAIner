import cv2
import mediapipe as mp
import sys
import lib.utils as utils
import json

arguments = sys.argv[1:]

videoFile = arguments[0]
destDir = arguments[1]
derive_features = utils.derive_features
derive_features_pushup = utils.derive_features_pushup
calculate_angle = utils.calculate_angle
calculate_distance = utils.calculate_distance
draw_XY_angle = utils.draw_XY_angle
mp_drawing = mp.solutions.drawing_utils
draw_Count = utils.draw_Count
mp_pose = mp.solutions.pose


####### Pose Detection #######
# Make Detection
cap = cv2.VideoCapture(videoFile)
pushup_count = 0
state_count = 0
S_angle = 100
confident_threshold = 0.5
smallestKneeAngle = S_angle
# Store an array of array of features for squatRec[i][j] represent ith squat and jth keyth points of the smallest knee angle frame
exerciseRec = [[]]
i = 0

width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))

outVideo = cv2.VideoWriter(destDir, cv2.VideoWriter_fourcc(
    *'avc1'), 30, (width, height), True)


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
            sl = [lm[pl.LEFT_SHOULDER.value].x, lm[pl.LEFT_SHOULDER.value].y]
            el = [lm[pl.LEFT_ELBOW.value].x, lm[pl.LEFT_ELBOW.value].y]
            wl = [lm[pl.LEFT_WRIST.value].x, lm[pl.LEFT_WRIST.value].y]

            sr = [lm[pl.RIGHT_SHOULDER.value].x, lm[pl.RIGHT_SHOULDER.value].y]
            er = [lm[pl.RIGHT_ELBOW.value].x, lm[pl.RIGHT_ELBOW.value].y]
            wr = [lm[pl.RIGHT_WRIST.value].x, lm[pl.RIGHT_WRIST.value].y]

            angleR = calculate_angle(sr, er, wr)
            angleL = calculate_angle(sl, el, wl)

            # draw_Count(image, "Count: ", pushup_count)
            # draw_XY_angle(image, "Count: ", "Left: ", "Right: ", pushup_count, angleL, angleR)
            draw_Count(image, "", pushup_count)

            # Frame Selection
            minElbow = min(angleR, angleL)

            if (minElbow < S_angle):  # down position
                if (state_count == 0):
                    state_count += 1

                if (minElbow < smallestKneeAngle):
                    smallestKneeAngle = minElbow
                    features_i = [lm[pl.NOSE.value], lm[pl.LEFT_SHOULDER.value], lm[pl.RIGHT_SHOULDER.value], lm[pl.LEFT_ELBOW.value], lm[pl.RIGHT_ELBOW.value], lm[pl.LEFT_WRIST.value],
                                  lm[pl.RIGHT_WRIST.value], lm[pl.LEFT_HIP.value], lm[pl.RIGHT_HIP.value], lm[pl.LEFT_KNEE.value], lm[pl.RIGHT_KNEE.value], lm[pl.LEFT_ANKLE.value], lm[pl.RIGHT_ANKLE.value]]
                    exerciseRec[i] = features_i

            elif (minElbow > S_angle):  # stand position
                if (state_count == 1):   # Already down position
                    state_count = 0
                    pushup_count += 1
                    i += 1
                    smallestKneeAngle = minElbow
                    exerciseRec.append([])

        except:
            pass

        # Write into frame
        outVideo.write(image)

    #     cv2.imshow('Mediapipe Feed', image)
    #     if cv2.waitKey(10) & 0xFF == ord('q'):
    #         break

    # cap.release()
    # cv2.destroyAllWindows()

# Output video
outVideo.release()

# ####### Data Processor #######
# row structure: {neck angle, elbow angle, hip angle, knee angle}
inputPreData = []
for exer_i_th in range(len(exerciseRec) - 1):  # last exercise is just uncounted
    df = derive_features_pushup(exerciseRec[exer_i_th])
    inputPreData.append(df)

# print(inputPreData)

####### Pose Classification #######
feedback = []
pushupClass = [
    {'name': 'Half Push-Up', 'description': 'Not enough bending to the ground.'},
    {'name': 'Bent Knee', 'description': 'Excessive bent on the knee.'},
    {'name': 'Tilted Neck', 'description': 'The head is tilted at the neck.'},
    {'name': 'Pelvis Curved', 'description': 'The pelvis is tilted up.'},
    {'name': 'Pelvis Dropped', 'description': 'The pelvis is bent down.'},
    {'name': 'Acceptable', 'description': 'Normal push-up'}
]

perfectSquat = [141.1026047751081, 52.83349409408348,
                175.2033315138845, 169.6284784841131]

# acceptedMultiplier = 0
acceptedAngleTolerance = 10
for exer_i_th in range(len(inputPreData)):
    groupClasses = []

    # Detecting all possible errors
    # Half pushup
    if (not (abs(inputPreData[exer_i_th][1] - perfectSquat[1]) <= acceptedAngleTolerance)):
        groupClasses.append(0)

    # Bent Knee
    if (inputPreData[exer_i_th][3] < perfectSquat[3] - acceptedAngleTolerance):
        groupClasses.append(1)

    # # Til
    # if(not(abs(inputPreData[exer_i_th][0] - acceptedAngleTolerance) < perfectSquat[0])):
    #     groupClasses.append(2)

    # HTB
    if (inputPreData[exer_i_th][2] < perfectSquat[2] - acceptedAngleTolerance):
        groupClasses.append(3)

    # HD
    if (inputPreData[exer_i_th][2] > 180):
        groupClasses.append(4)

    # ACC
    if (len(groupClasses) == 0):
        groupClasses.append(5)

    ######## Feedback Generator #######
    fb_i = {
        'ex_number': exer_i_th + 1,
        'class': "",
        'correction': ""
    }
    for i in range(len(groupClasses)):
        if (i == len(groupClasses) - 1):
            fb_i['class'] += pushupClass[groupClasses[i]]['name']
            fb_i['correction'] += pushupClass[groupClasses[i]]['description']
        else:
            fb_i['class'] += pushupClass[groupClasses[i]]['name'] + ', '
            fb_i['correction'] += pushupClass[groupClasses[i]]['description'] + ' '

    feedback.append(fb_i)

print(json.dumps(feedback))
