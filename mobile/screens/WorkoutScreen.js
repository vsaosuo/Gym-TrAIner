import { StyleSheet, Text, View, FlatList, RefreshControl, SafeAreaView, ScrollView } from 'react-native'
import React, { useEffect, useState, useRef, useCallback } from 'react'
import { Video } from 'expo-av'
import { storage } from '../firebase'
import { formatDateString, formatTimestampDate, getTimeTimestamp, capitalizeString, calculateSquatFeedback, calculateAccuracyPercentage, calculatePushupFeedback } from '../utils'
import { Ionicons } from '@expo/vector-icons'; 
import { database, auth } from '../firebase'
import { getPushupClassColor, getSquatClassColor } from '../utils';

const WorkoutScreen = ({route, navigation}) => {
    const { id, date, type } = route.params;

    const video = useRef(null);
    const [videoUrl, setVideoUrl] = useState(null);
    const [reps, setReps] = useState([]);
    const [videoId, setVideoId] = useState("");
    const [refreshing, setRefreshing] = useState(false);

    //Initial load of reps and video ID
    useEffect(() => {
      queryData();
    }, []);

    const onRefresh = useCallback(() => {
      setRefreshing(true);
      queryData();
      setRefreshing(false);
    }, []);

    const queryData = () => {
      database.collection('users').doc(auth.currentUser?.uid).collection('workouts').doc(id).get().then((doc) => {
        const workoutInfo = doc.data();
        setReps(workoutInfo.reps);
        setVideoId(workoutInfo.video_id);

        if (workoutInfo.video_id !== videoId) {
          setVideoId(workoutInfo.video_id);
          var videoRef = storage.ref(`videos/${workoutInfo.video_id}`);
          videoRef.getDownloadURL()
          .then((url) => {
              setVideoUrl(`${url}.mp4`);
          })
        }
      })
      .catch((error) => alert(error));
    }

    return (
        <ScrollView
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
        }>
          <View style={styles.container}>
            <View style={styles.workoutSummaryHeader}>
                <Ionicons name="arrow-back-circle-outline" size={34} color='white' style={styles.backButton} 
                          onPress={() => navigation.goBack()}
                />
                <View style={styles.workoutSummaryHeaderInfo}>
                  <Text style={styles.workoutHeaderText}>{`${capitalizeString(type)} exercise`}</Text>
                  <Text style={styles.workoutDateText}>{`${formatDateString(formatTimestampDate(date))}`}</Text>
                  <Text style={styles.workoutTimeText}>{`${getTimeTimestamp(date)}`}</Text>
                </View>
            </View>
            {videoUrl && 
            <Video
                ref={video}
                style={styles.video}
                source={{
                    uri: videoUrl,
                }}
                useNativeControls
                resizeMode="contain"
                isLooping
            />
            }
            {!videoUrl && 
            <View style={styles.videoLoading}>
              <Text>Video pending upload from server...</Text>
            </View>}
            <View style={styles.summary}>
              <Text style={styles.summaryHeaderText}>{type == 'squat' ? calculateSquatFeedback(reps) : calculatePushupFeedback(reps)}</Text>
            </View>
              <View style={styles.repListContainer}>
                {reps &&
                reps.map((rep, index) => {
                  return (
                    <View style={styles.listItem}
                      key={index}
                    >
                    <View style={styles.listHeader}>
                      <View style={styles.listItemNumberCircle}>
                        <Text style={styles.listItemNumberText}>{index + 1}</Text>
                      </View>
                      <View style={styles.listItemText}>
                      {rep.class.split(", ").map((className, index2) => {
                        return (
                        <Text
                        key={index2} 
                        style={{
                          fontSize: 18,
                          color: type === "squat" ? getSquatClassColor(className) : getPushupClassColor(className),
                        }}>{index2 === rep.class.split(", ").length - 1 ? className : className + ", "}
                        </Text>
                        )
                      })}
                      </View> 
                    </View>
                    <Text style={styles.correctionText}>{rep.correction}</Text>
                  </View>
                  )
                })
                }
              </View>
            </View>
          </ScrollView>
    )
}

export default WorkoutScreen

const styles = StyleSheet.create({
    listItem: {
      backgroundColor: 'white',
    },
    listItemText: {
      flexWrap: 'wrap',
      display: 'flex',
    },
    correctionText: {
      marginLeft: 10,
      marginBottom: 5,
    },
    listHeader: {
      flexDirection: 'row',
      alignItems: 'center',
      padding: 10,
    },
    listItemNumberCircle: {
      width: 30,
      height: 30,
      justifyContent: 'center',
      borderRadius: 30 / 2,
      backgroundColor: '#0782F9',
      marginRight: 10,
    },
    listItemNumberText: {
      alignSelf: 'center',
      fontWeight: 'bold',
      color: 'white',
      fontSize: 15,
    },
    summary: {
      alignItems: 'center',
      borderRadius: 10,
      width: 412,
      height: 50,
      backgroundColor: '#0782F9',
    },
    summaryHeaderText: {
      color: 'white',
      fontWeight: '700',
      fontSize: 22,
      marginTop: 10,
    },
    repListContainer: {
      flex: 1,
      flexGrow: 1,
      width: '100%',
      marginBottom: 10,
    },
    container: {
      margionTop: -30,
      flex: 1,
      alignItems: 'center',
      width: '100%'
    },
    header: {
      margin: 25
    },
    video: {
      alignSelf: 'center',
      width: '100%',
      height: 500,
      borderRadius: 10,
    },
    videoLoading: {
      alignItems: 'center',
      justifyContent: 'center',
      width: '100%',
      height: '30%',
      borderRadius: 10,
    },
    workoutSummaryHeader: {
      marginTop: 60,
      display: 'flex',
      alignItems: 'center',
      flexWrap: 'nowrap',
      borderRadius: 10,
      width: '100%',
      height: 100,
      backgroundColor: '#0782F9',
    },
    workoutSummaryHeaderInfo: {
      alignItems: 'center'
    },
    backButton: {
      marginTop: 25,
      flex: '0 1 auto',
      position: 'absolute',
      left: '10%'
    },
    workoutHeaderText: {
      color: 'white',
      fontWeight: '700',
      fontSize: 24,
      marginTop: 5,
    },
    workoutDateText: {
      color: 'white',
      fontWeight: '700',
      fontSize: 20,
      marginTop: 5,
    },
    workoutTimeText: {
      color: 'white',
      fontWeight: '700',
      fontSize: 16,
      marginTop: 5,
    },
    feedbackContainer: {
      alignItems: 'center',
      width: '95%',
      height: '30%',
      backgroundColor: '#0782F9',
      borderRadius: 10,
    },
    accuracyText: {
      fontSize: 35,
    },
    feedbackText: {
      color: 'white',
      fontSize: 20,
    },
})