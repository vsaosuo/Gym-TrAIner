import { StyleSheet, Text, View, TouchableOpacity, ActivityIndicator } from 'react-native';
import React, { useEffect, useState } from 'react';
import { auth, database } from '../firebase';
import { useNavigation } from '@react-navigation/native';

const HomeScreen = () => {
  const [name, setName] = useState('');
  const [loading, setLoading] = useState(false);

  const navigation = useNavigation();

  useEffect(() => {
    setLoading(true);
    database.collection('users').doc(auth.currentUser?.uid).get().then((userDoc) => {
      setName(userDoc.data().name);
      setLoading(false);
    })
  }, []);

  const handleSignOut = () => {
    auth
      .signOut()
      .then(() => {
        navigation.replace("Login")
      })
      .catch(error => alert(error.message))
  }

  return (
    <View style={styles.container}>
      {loading && 
      <ActivityIndicator size="large" color="#0782F9"/>}
      {!loading && 
      <>
        <Text style={styles.headerText}>Welcome, { name }.</Text>
        <View style={styles.buttonContainer}>
          <TouchableOpacity
            style={styles.button}
            onPress={() => navigation.navigate("WorkoutSummary")}
          >
            <Text style={styles.buttonText}>View Workouts</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={styles.button}
            onPress={() => navigation.navigate("QRScanner")}
          >
            <Text style={styles.buttonText}>Link Device</Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.button, styles.buttonOutline]}
            onPress={handleSignOut}
          >
            <Text style={styles.buttonOutlineText}>Sign out</Text>
          </TouchableOpacity>
        </View>
      </>}
    </View>
  )
}

export default HomeScreen

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center'
  },
  headerText: {
    color: '#0782F9',
    fontWeight: '700',
    fontSize: 30,
    marginTop: 40
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