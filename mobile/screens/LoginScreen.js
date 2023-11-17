import { StyleSheet, Text, TextInput, TouchableOpacity, View, Image } from 'react-native'
import { useEffect } from 'react';
import React from 'react'
import { useState } from 'react';
import { auth } from '../firebase';
import { useNavigation } from '@react-navigation/native';

const LoginScreen = () => {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');

    const navigation = useNavigation();

    useEffect(() => {
        const unsubscribe = auth.onAuthStateChanged(user => {
            if (user) {
                navigation.navigate("Home");
            }
        })
        return unsubscribe
    }, [])

    const handleLogin = () => {
        auth
        .signInWithEmailAndPassword(email, password)
        .catch(error => alert(error.message));
    }

  return (
    <View
        style={styles.container}
        behavior="padding"
    >
        <View style={styles.header}>
            <Image source={require('../assets/temp-logo.png')}
            style={{width: 250,
                    height: 250,
                    marginLeft: 19,
                    marginTop: -50,}}
            />
            <Text style={styles.headerText}>
                Login to Get Started
            </Text>
        </View>
        <View style={styles.inputContainer}>
            <TextInput
                placeholder='Email'
                value={email}
                onChangeText={text => setEmail(text)}
                style={styles.input}
            />
            <TextInput
                placeholder='Password'
                value={password}
                onChangeText={text => setPassword(text)}
                style={styles.input}
                secureTextEntry
            />
        </View>

        <View style={styles.buttonContainer}>
            <TouchableOpacity
                onPress={handleLogin}
                style={styles.button}
            >
                <Text style={styles.buttonText}>Login</Text>
            </TouchableOpacity>
            <TouchableOpacity
                onPress={() => navigation.navigate("Register")}
                style={[styles.button, styles.buttonOutline]}
            >
                <Text style={styles.buttonOutlineText}>Register</Text>
            </TouchableOpacity>
        </View>
    </View>
  )
}

export default LoginScreen

const styles = StyleSheet.create({
    container: {
        flex: 1,
        justifyContent: 'center',
        alignItems: 'center',
    },
    headerText: {
        color: '#0782F9',
        fontWeight: '700',
        fontSize: 30,
        marginTop: 40
    },
    inputContainer: {
        width: '80%',
        marginTop: 40,
    },
    input: {
        backgroundColor: 'white',
        paddingHorizontal: 15,
        paddingVertical: 10,
        borderRadius: 10,
        marginTop: 5,
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
});