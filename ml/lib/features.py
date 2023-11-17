import os
import glob
import pandas as pd

import utils
calculate_angle = utils.calculate_angle
calculate_distance = utils.calculate_distance

####### Functions derived from Renzo John Virtucio #######
def derive_features(dframe):
  df = pd.DataFrame()
  for f in dframe:
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

    df = df.append(row, ignore_index=True)

  df = dframe.drop_duplicates()
  return df