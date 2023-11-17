import { StyleSheet, Text, View, ActivityIndicator, TouchableOpacity, RefreshControl, SafeAreaView, ScrollView } from 'react-native'
import React, { useState, useEffect, useCallback } from 'react'
import { Calendar } from 'react-native-calendars'
import { formatDateString, formatTimestampDate, getTimeTimestamp, capitalizeString } from '../utils'
import { database, auth } from '../firebase'
import { useNavigation } from '@react-navigation/native'
import { Ionicons } from '@expo/vector-icons'; 

const WorkoutSummaryScreen = () => {
    const [selectedDate, setSelectedDate] = useState(null);
    const [markedDates, setMarkedDates] = useState();
    const [workoutSummary, setWorkoutSummary] = useState([]);
    const [loading, setLoading] = useState(false);
    const [refreshing, setRefreshing] = useState(false);

    const navigation = useNavigation();

    //Initialize screen 
    useEffect(() => {
        setLoading(true);
        initWorkoutInformation();
        setLoading(false);
    }, [])
    
    const onRefresh = useCallback(() => {
      setRefreshing(true);
      initWorkoutInformation();
      setRefreshing(false);
    }, []);

    // Make call to Firestore database and get all workout information
    const initWorkoutInformation = () => {
      const workoutSummary = [];
      database.collection('users').doc(auth.currentUser?.uid).collection('workouts').orderBy('date').get().then((querySnapshot) => {
          querySnapshot.forEach((doc) => {
              const workoutInfo = doc.data();
              workoutInfo['id'] = doc.id;
              workoutSummary.push(workoutInfo);
          })
          // Set initial state
          const markedDates = {};
          for (let i = 0; i < workoutSummary.length; i++) {
              markedDates[formatTimestampDate(workoutSummary[i].date)] = { marked: true }
          }
          const today = new Date();
          today.setDate(today.getDate());
          setSelectedDate(today);
          const todayFormatted = today.toISOString().split('T')[0]
          markedDates[todayFormatted] = { selected: true, marked: markedDates.hasOwnProperty(todayFormatted) && markedDates[todayFormatted].marked };

          setWorkoutSummary(workoutSummary);
          setMarkedDates(markedDates);
      })
      .catch((error) => alert(error));
  }

    const selectDate = (date) => {
        setSelectedDate(date);
        let markedDatesCopy = {...markedDates}
        // Reset selected dates
        Object.keys(markedDatesCopy).forEach(function(day) {
            const isMarked = markedDatesCopy.hasOwnProperty(day) && markedDatesCopy[day].marked;
            markedDatesCopy[day] = { selected: false, marked: isMarked };
        });

        // Selected date and set marked dates
        const isMarked = markedDatesCopy.hasOwnProperty(date) && markedDatesCopy[date].marked;
        markedDatesCopy[date] = { selected: true, marked: isMarked };
        setMarkedDates(markedDatesCopy);
    }

    if (loading) {
        return (
            <View style={styles.container}>
                <ActivityIndicator size="large" color="#0782F9"/>
            </View>
        )
    }

    return (
        <ScrollView
          contentContainerStyle={styles.scrollView}
          refreshControl={
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          }>
          <View style={styles.container}>
            <View style={styles.header}>
                <Ionicons name="arrow-back-circle-outline" size={34} color='#0782F9' style={styles.backButton} 
                          onPress={() => navigation.goBack()}
                />
                <Text style={styles.headerText}>Workout Calendar</Text>
            </View>
            <Calendar
            style={styles.calendarStyle}
            theme={{
            todayTextColor: '#0782F9',
            textDayFontSize: 16,
            textMonthFontSize: 24,
            textDayHeaderFontSize: 16,
            selectedDayBackgroundColor: '#0782F9',
            selectedDayTextColor: '#ffffff',
            }}
            markedDates={markedDates}
            onDayPress={day => selectDate(day.dateString)}
            />
            <View style={styles.workoutSummaryContainer}>
                <View style={styles.workoutSummaryHeader}>
                    <Text style={styles.workoutHeaderText}>{formatDateString(selectedDate)}</Text>
                </View>
                <View style={styles.workoutListContainer}>
                    {workoutSummary.filter((workout) => {
                        return formatTimestampDate(workout.date) === selectedDate;
                    }).map((workout, index) => {
                        return (
                            <TouchableOpacity
                              key={index}
                              style={workout.type == "squat" ? styles.workoutItem : styles.workoutItemAlt}
                              onPress={() => navigation.navigate('Workout', {
                                id: workout.id,
                                reps: workout.reps,
                                date: workout.date,
                                type: workout.type,
                                videoId: workout.video_id,
                              })}
                              >
                                <View style={styles.workoutTimeContainer}>
                                    <Text style={styles.workoutTime}>{getTimeTimestamp(workout.date)}</Text>
                                </View>
                                <Text style={styles.workoutNameText}>{capitalizeString(workout.type)}</Text>
                            </TouchableOpacity>
                        )
                    })
                    }
                </View>
            </View>
          </View>
        </ScrollView>
    )
}

export default WorkoutSummaryScreen

const styles = StyleSheet.create({
    container: {
        flex: 1,
        alignItems: 'center',
        width: '100%',
      },
      header: {
        display: 'flex',
        alignItems: 'center',
        flexWrap: 'nowrap',
        margin: 25
      },
      backButton: {
        marginTop: 40,
        flex: '0 1 auto',
        position: 'absolute',
        left: '-15%'
      },
      calendarStyle: {
        borderRadius: 10,
        width: 410,
        height: 350
      },
      headerText: {
        color: '#0782F9',
        fontWeight: '700',
        fontSize: 30,
        marginTop: 40,
        right: '-2%'
      },
      workoutSummaryContainer: {
        flex: 1,
        alignItems: 'center',
        backgroundColor: 'white',
        width: 410,
        borderRadius: 10
      },
      workoutSummaryHeader: {
        alignItems: 'center',
        borderTopLeftRadius: 10,
        borderTopRightRadius: 10,
        width: 412,
        height: 50,
        backgroundColor: '#0782F9',
      },
      workoutHeaderText: {
        color: 'white',
        fontWeight: '700',
        fontSize: 24,
        marginTop: 7,
      },
      workoutListContainer: {
        height: 360,
        marginBottom: 10,
      },
      workoutItem: {
        flexDirection: 'row',
        height: 60,
        width: 412,
        backgroundColor: '#89cff0',
        alignItems: 'center',
        borderWidth: 1,
        borderColor: 'black',
        borderTopWidth: 0,
      },
      workoutItemAlt: {
        flexDirection: 'row',
        height: 60,
        width: 412,
        backgroundColor: '#D3D3D3',
        alignItems: 'center',
        borderWidth: 1,
        borderTopWidth: 0,
        borderColor: 'black',
      },
      workoutTimeContainer: {
        height: '100%',
        width: 100,
        borderRightWidth: 1,
        borderColor: 'black',
        alignItems: 'center',
      },
      workoutNameText: {
        fontSize: 18,
        marginLeft: 15,
      },
      workoutTime: {
        marginTop: 15,
        fontSize: 18,
      }, 
      buttonContainer: {
        width: '60%',
        justifyContent: 'center',
        alignItems: 'center',
        marginTop: 40,
      },
      button: {
        backgroundColor: '#0782F9',
        width: '100%',
        marginTop: 5,
        padding: 15,
        borderRadius: 10,
        alignItems: 'center',
      },
      buttonOutline: {
        backgroundColor: 'white',
        marginTop: 5,
        borderColor: '#0782F9',
        borderWidth: 2,
      },
      buttonText: {
          color: 'white',
          fontWeight: '700',
          fontSize: 16,
      },
      buttonOutlineText: {
          color: '#0782F9',
          fontWeight: '700',
          fontSize: 16,
      },
})