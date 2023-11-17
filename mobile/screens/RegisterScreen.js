import { StyleSheet, Text, View, TextInput, TouchableOpacity } from 'react-native'
import { useState } from 'react'
import React from 'react'
import { useNavigation } from '@react-navigation/native';
import { auth, database } from '../firebase'

const RegisterScreen = () => {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [name, setName] = useState('');

    const navigation = useNavigation();

    const handleSignUp = () => {
        auth
        .createUserWithEmailAndPassword(email, password)
        .then(userCredentials => {
            database.collection('users').doc(userCredentials.user.uid).set({
                email: email,
                password: password,
                name: name,
            })
        })
        .then(() => {
            navigation.navigate("Login");
        })
        .catch(error => alert(error.message));
    }

  return (
    <View style={styles.container}>
        <View style={styles.header}>
            <Text style={styles.headerText}>Register for an Account</Text>
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
            <TextInput
                placeholder='Name'
                value={name}
                onChangeText={text => setName(text)}
                style={styles.input}
            />
        </View>
        <View style={styles.buttonContainer}>
            <TouchableOpacity
                onPress={handleSignUp}
                style={[styles.button, styles.button]}
            >
                <Text style={styles.buttonText}>Register</Text>
            </TouchableOpacity>
        </View>
        <View style={styles.signInTextContainer}>
            <Text style={styles.signInText}>Already have an account? </Text>
            <Text 
                style={[styles.signInLinkText, styles.signInText]}
                onPress={() => navigation.navigate("Login")}
            >
                Sign in.
            </Text>
        </View>
    </View>
  )
}

export default RegisterScreen

const styles = StyleSheet.create({
    container: {
        flex: 1,
        justifyContent: 'center',
        alignItems: 'center',
    },
    header: {
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
    signInTextContainer: {
        marginTop: 10,
        flexDirection: 'row',
    },
    signInText: {
        fontWeight: 'bold'
    },
    signInLinkText: {
        textDecorationLine: 'underline',
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
    buttonText: {
        color: 'white',
        fontWeight: '700',
        fontSize: 16,
    },
})