import cv2
import math
import numpy as np
import pandas as pd

####### Functions import from Renzo John Virtucio #######
def rescale_frame(frame, percent=75):
  width = int(frame.shape[1] * percent/ 100)
  height = int(frame.shape[0] * percent/ 100)
  dim = (width, height)
  return cv2.resize(frame, dim, interpolation =cv2.INTER_AREA)

def calculate_angle(a,b,c):
  a = np.array(a) # First
  b = np.array(b) # Mid
  c = np.array(c) # End
  
  radians = np.arctan2(c[1]-b[1], c[0]-b[0]) - np.arctan2(a[1]-b[1], a[0]-b[0])
  angle = np.abs(radians*180.0/np.pi)
  
  if angle >180.0:
    angle = 360-angle
      
  return angle 

def calculate_distance(a, b):
  a = np.array(a) # Start
  b = np.array(b) # End

  dist = np.linalg.norm(a - b)
  
  return dist

def resize_with_aspect_ratio(image, width=None, height=None, inter=cv2.INTER_AREA):
    dim = None
    (h, w) = image.shape[:2]

    if width is None and height is None:
        return image
    if width is None:
        r = height / float(h)
        dim = (int(w * r), height)
    else:
        r = width / float(w)
        dim = (width, int(h * r))

    return cv2.resize(image, dim, interpolation=inter)

####### Functions derived from Renzo John Virtucio #######
# input a list and output a dataframe
def derive_features_df(listData):
  df = pd.DataFrame()
  for f in listData:
    row = {}

    row['left_knee_angle'] = calculate_angle([f[2].x,f[2].y], [f[4].x,f[4].y], [f[6].x,f[6].y])
    row['left_hip_angle'] = calculate_angle([f[0].x,f[0].y], [f[2].x,f[2].y], [f[4].x,f[4].y])
    row['left_ankle_angle'] = calculate_angle([f[4].x,f[4].y], [f[6].x,f[6].y], [f[10].x,f[10].y])
    
    row['right_knee_angle'] = calculate_angle([f[3].x,f[3].y], [f[5].x,f[5].y], [f[7].x,f[7].y])
    row['right_hip_angle'] = calculate_angle([f[1].x,f[1].y], [f[3].x,f[3].y], [f[5].x,f[5].y])
    row['right_ankle_angle'] = calculate_angle([f[5].x,f[5].y], [f[7].x,f[7].y], [f[11].x,f[11].y])
    
    row['left_hip_to_right_hip'] = calculate_distance([f[2].x,f[2].y], [f[3].x,f[3].y])

    row['left_torso_height_to_hip_width_ratio'] = calculate_distance([f[0].x,f[0].y], [f[2].x,f[2].y])/row['left_hip_to_right_hip']
    row['right_torso_height_to_hip_width_ratio'] = calculate_distance([f[1].x,f[1].y], [f[3].x,f[3].y])/row['left_hip_to_right_hip']
    row['shoulder_width_to_hip_width_ratio'] = calculate_distance([f[0].x,f[0].y], [f[1].x,f[1].y])/row['left_hip_to_right_hip']

    row['left_thigh_height_to_hip_width_ratio'] = calculate_distance([f[2].x,f[2].y], [f[4].x,f[4].y])/row['left_hip_to_right_hip']
    row['right_thigh_height_to_hip_width_ratio'] = calculate_distance([f[3].x,f[3].y], [f[5].x,f[5].y])/row['left_hip_to_right_hip']
    row['left_shin_length_to_hip_width_ratio'] = calculate_distance([f[4].x,f[4].y], [f[6].x,f[6].y])/row['left_hip_to_right_hip']
    row['right_shin_length_to_hip_width_ratio'] = calculate_distance([f[5].x,f[5].y], [f[7].x,f[7].y])/row['left_hip_to_right_hip']

    row['left_ankle_to_left_heel_to_hip_width_ratio'] = calculate_distance([f[6].x,f[6].y], [f[8].x,f[8].y])/row['left_hip_to_right_hip']
    row['left_heel_to_left_foot_index_to_hip_width_ratio'] = calculate_distance([f[8].x,f[8].y], [f[10].x,f[10].y])/row['left_hip_to_right_hip']
    row['left_foot_index_to_left_ankle_to_hip_width_ratio'] = calculate_distance([f[10].x,f[10].y], [f[6].x,f[6].y])/row['left_hip_to_right_hip']

    row['right_ankle_to_right_heel_to_hip_width_ratio'] = calculate_distance([f[7].x,f[7].y], [f[9].x,f[9].y])/row['left_hip_to_right_hip']
    row['right_heel_to_right_foot_index_to_hip_width_ratio'] = calculate_distance([f[9].x,f[9].y], [f[11].x,f[11].y])/row['left_hip_to_right_hip']
    row['right_foot_index_to_right_ankle_to_hip_width_ratio'] = calculate_distance([f[11].x,f[11].y], [f[7].x,f[7].y])/row['left_hip_to_right_hip']

    row['knee_width_to_hip_width_ratio'] = calculate_distance([f[4].x,f[4].y], [f[5].x,f[5].y])/row['left_hip_to_right_hip']

    df = pd.concat([df, pd.DataFrame(row, index=[0])], ignore_index=True)

  df = df.drop_duplicates()
  return df

# def derive_features(listData):
#   df = []
#   for f in listData:
#     row = []

#     row.append(calculate_angle([f[2].x,f[2].y], [f[4].x,f[4].y], [f[6].x,f[6].y]))
#     row.append(calculate_angle([f[0].x,f[0].y], [f[2].x,f[2].y], [f[4].x,f[4].y]))
#     row.append(calculate_angle([f[4].x,f[4].y], [f[6].x,f[6].y], [f[10].x,f[10].y]))
    
#     row.append(calculate_angle([f[3].x,f[3].y], [f[5].x,f[5].y], [f[7].x,f[7].y]) )
#     row.append(calculate_angle([f[1].x,f[1].y], [f[3].x,f[3].y], [f[5].x,f[5].y]) )
#     row.append(calculate_angle([f[5].x,f[5].y], [f[7].x,f[7].y], [f[11].x,f[11].y]))
    
#     row.append(calculate_distance([f[2].x,f[2].y], [f[3].x,f[3].y]))

#     row.append(calculate_distance([f[0].x,f[0].y], [f[2].x,f[2].y])/row[6])
#     row.append(calculate_distance([f[1].x,f[1].y], [f[3].x,f[3].y])/row[6])
#     row.append(calculate_distance([f[0].x,f[0].y], [f[1].x,f[1].y])/row[6])

#     row.append(calculate_distance([f[2].x,f[2].y], [f[4].x,f[4].y])/row[6])
#     row.append(calculate_distance([f[3].x,f[3].y], [f[5].x,f[5].y])/row[6])
#     row.append(calculate_distance([f[4].x,f[4].y], [f[6].x,f[6].y])/row[6])
#     row.append(calculate_distance([f[5].x,f[5].y], [f[7].x,f[7].y])/row[6])

#     row.append(calculate_distance([f[6].x,f[6].y], [f[8].x,f[8].y])/row[6])
#     row.append(calculate_distance([f[8].x,f[8].y], [f[10].x,f[10].y])/row[6])
#     row.append(calculate_distance([f[10].x,f[10].y], [f[6].x,f[6].y])/row[6])

#     row.append(calculate_distance([f[7].x,f[7].y], [f[9].x,f[9].y])/row[6])
#     row.append(calculate_distance([f[9].x,f[9].y], [f[11].x,f[11].y])/row[6])
#     row.append(calculate_distance([f[11].x,f[11].y], [f[7].x,f[7].y])/row[6])

#     row.append(calculate_distance([f[4].x,f[4].y], [f[5].x,f[5].y])/row[6])

#     df.append(row)
    
#   return df

def derive_features(f):
  row = []

  row.append(calculate_angle([f[2].x,f[2].y], [f[4].x,f[4].y], [f[6].x,f[6].y]))
  row.append(calculate_angle([f[0].x,f[0].y], [f[2].x,f[2].y], [f[4].x,f[4].y]))
  row.append(calculate_angle([f[4].x,f[4].y], [f[6].x,f[6].y], [f[10].x,f[10].y]))
  
  row.append(calculate_angle([f[3].x,f[3].y], [f[5].x,f[5].y], [f[7].x,f[7].y]) )
  row.append(calculate_angle([f[1].x,f[1].y], [f[3].x,f[3].y], [f[5].x,f[5].y]) )
  row.append(calculate_angle([f[5].x,f[5].y], [f[7].x,f[7].y], [f[11].x,f[11].y]))
  
  row.append(calculate_distance([f[2].x,f[2].y], [f[3].x,f[3].y]))

  row.append(calculate_distance([f[0].x,f[0].y], [f[2].x,f[2].y])/row[6])
  row.append(calculate_distance([f[1].x,f[1].y], [f[3].x,f[3].y])/row[6])
  row.append(calculate_distance([f[0].x,f[0].y], [f[1].x,f[1].y])/row[6])

  row.append(calculate_distance([f[2].x,f[2].y], [f[4].x,f[4].y])/row[6])
  row.append(calculate_distance([f[3].x,f[3].y], [f[5].x,f[5].y])/row[6])
  row.append(calculate_distance([f[4].x,f[4].y], [f[6].x,f[6].y])/row[6])
  row.append(calculate_distance([f[5].x,f[5].y], [f[7].x,f[7].y])/row[6])

  row.append(calculate_distance([f[6].x,f[6].y], [f[8].x,f[8].y])/row[6])
  row.append(calculate_distance([f[8].x,f[8].y], [f[10].x,f[10].y])/row[6])
  row.append(calculate_distance([f[10].x,f[10].y], [f[6].x,f[6].y])/row[6])

  row.append(calculate_distance([f[7].x,f[7].y], [f[9].x,f[9].y])/row[6])
  row.append(calculate_distance([f[9].x,f[9].y], [f[11].x,f[11].y])/row[6])
  row.append(calculate_distance([f[11].x,f[11].y], [f[7].x,f[7].y])/row[6])

  row.append(calculate_distance([f[4].x,f[4].y], [f[5].x,f[5].y])/row[6])
    
  return row

def derive_features_pushup(f):
  row = []

  # row structure: {neck angle, elbow angle, hip angle, knee angle}
  # f structure: {nose, shoulder L/R, elbow L/R, wrist L/R, hip L/R,  knee L/R, ankle L/R }

  a = 0 # nose
  b = 1 if f[1].visibility >= f[2].visibility else 2 # shoulder
  c = 7 if f[7].visibility >= f[8].visibility else 8 # hip

  row.append(calculate_angle([f[a].x,f[a].y], [f[b].x,f[b].y], [f[c].x,f[c].y]))

  a = 1 if f[1].visibility >= f[2].visibility else 2 # shoulder
  b = 3 if f[3].visibility >= f[3].visibility else 3 # Elbow
  c = 5 if f[5].visibility >= f[6].visibility else 6 # Wrist
  row.append(calculate_angle([f[a].x,f[a].y], [f[b].x,f[b].y], [f[c].x,f[c].y]))

  a = 1 if f[1].visibility >= f[2].visibility else 2 # shoulder
  b = 7 if f[7].visibility >= f[8].visibility else 8 # hip
  c = 9 if f[9].visibility >= f[10].visibility else 10 # knee
  row.append(calculate_angle([f[a].x,f[a].y], [f[b].x,f[b].y], [f[c].x,f[c].y]))

  a = 7 if f[7].visibility >= f[8].visibility else 8 # hip
  b = 9 if f[9].visibility >= f[10].visibility else 10 # knee
  c = 11 if f[11].visibility >= f[12].visibility else 11 # ankle
  row.append(calculate_angle([f[a].x,f[a].y], [f[b].x,f[b].y], [f[c].x,f[c].y]))
    
  return row


####### Functions Derived From Nicholas Renotte Tutorials #######

# Draw X and Y axise on the frame
def draw_XY_angle(frame, text_a1, text_a2, text_a3, a1, a2, a3):
  # Display angle
  cv2.putText(frame, text_a3 + str(round(a3)), (0, 100), cv2.FONT_HERSHEY_SIMPLEX, 2, (0, 0, 255), 3, cv2.LINE_AA)
  cv2.putText(frame, text_a1 + str(round(a1)), (0, 200), cv2.FONT_HERSHEY_SIMPLEX, 2, (0, 0, 255), 3, cv2.LINE_AA)
  cv2.putText(frame, text_a2 + str(round(a2)), (0, 300), cv2.FONT_HERSHEY_SIMPLEX, 2, (0, 0, 255), 3, cv2.LINE_AA)

# Draw X and Y axise on the frame
def draw_Count(frame, text_a1, a1):
  # Display angle
  cv2.putText(frame, text_a1 + str(round(a1)), (0, 100), cv2.FONT_HERSHEY_SIMPLEX, 1, (0, 0, 255), 3, cv2.LINE_AA)
  