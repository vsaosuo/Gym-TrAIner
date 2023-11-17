import { StyleSheet, Text, View, TouchableOpacity, ActivityIndicator } from 'react-native';
import React, { useEffect, useState } from 'react';
import { BarCodeScanner } from 'expo-barcode-scanner';
import { useNavigation } from '@react-navigation/native';
import { auth } from '../firebase'

const IP_ADDRESS = "206.87.197.46:3000"

const QRScannerScreen = () => {
    const [webSocket, setWebSocket] = useState(null);
    const [hasPermission, setHasPermission] = useState(false);
    const [scanData, setScanData] = useState();
    const [loading, setLoading] = useState(false);
    const [link, setLink] = useState(null);
    const [init, setInit] = useState(true);
    
    const navigation = useNavigation();
  
    useEffect(() => {
        (async() => {
            const {status} = await BarCodeScanner.requestPermissionsAsync();
            setHasPermission(status === "granted");
        })();
    }, []);

    const handleDisconnect = () => {
        const jsonDisconnectMessage = {
            type: 'disconnect',
        }
        webSocket.send(JSON.stringify(jsonDisconnectMessage));
        webSocket.close();
        setScanData(null);
        setInit(true);
        setLink(null);
        setWebSocket(null);
    }

    useEffect(() => {
        return () => {
            if (webSocket){
                handleDisconnect();
            }
        }
    }, [])
  
    if (!hasPermission) {
        return (
            <View style={styles.container}>
                <Text>Please grant camera permissions to app.</Text>
            </View>
        );
    }
  
    const handleBarCodeScanned = ({type, data}) => {
        setScanData(data);
    };


    const linkDevice = () => {
        setLoading(true);
        const timeout = setTimeout(() => {
            alert("Device ID not found, try again");
            setLoading(false);
        }, 5000);
        var ws = new WebSocket(`ws://${IP_ADDRESS}/user?id=${auth.currentUser?.uid}`)
        ws.onopen = () => {
            setWebSocket(ws);
            clearTimeout(timeout);
            const jsonConnectMessage = {
                device_id: scanData,
                type: 'connect',
            }
            ws.send(JSON.stringify(jsonConnectMessage));
            setLoading(false);
            setLink(scanData);
        }

        ws.onerror = e => {
            ws.close();
            console.log(e.message);
        }

        ws.onclose = e => {
            console.log(e.code, e.reason);
        }
        setWebSocket(ws);
    }

    if (loading) {
        return (
        <View style={styles.container}>
            <ActivityIndicator size="large" color="#0782F9"/>
        </View>
        )
    }

    if (init) {
        return (
        <View style={styles.container}>
            <View style={styles.buttonContainer}>
                <TouchableOpacity
                    onPress={() => setInit(false)}
                    style={styles.button}
                >
                    <Text style={styles.buttonText}>Scan device</Text>
                </TouchableOpacity>
                <TouchableOpacity
                    onPress={() => navigation.goBack()}
                    style={[styles.button, styles.buttonOutline]}
                >
                    <Text style={styles.buttonOutlineText}>Return to home</Text>
                </TouchableOpacity>
            </View>
        </View>
        )
    }

    if (link) {
        return (
        <View style={styles.container}>
            <Text style={styles.headerText}>Succesfully linked with Tr-AI-Ner:</Text>
            <View style={styles.deviceTextContainer}>
                <Text style={styles.deviceText}>{scanData}</Text>
            </View>
            <View style={styles.buttonContainer}>
                <TouchableOpacity
                    onPress={handleDisconnect}
                    style={styles.button}
                >
                    <Text style={styles.buttonText}>Disconnect</Text>
                </TouchableOpacity>
                <TouchableOpacity
                    onPress={() => navigation.navigate("Home")}
                    style={[styles.button, styles.buttonOutline]}
                >
                    <Text style={styles.buttonOutlineText}>Return to home</Text>
                </TouchableOpacity>
            </View>
        </View>
        )
    }

    if (scanData) {
        return (
        <View style={styles.container}>
            <Text style={styles.headerText}>Scanned device:</Text>
            <View style={styles.deviceTextContainer}>
                <Text style={styles.deviceText}>{scanData}</Text>
            </View>
            <View style={styles.buttonContainer}>
                <TouchableOpacity
                    onPress={linkDevice}
                    style={styles.button}
                >
                    <Text style={styles.buttonText}>Link device</Text>
                </TouchableOpacity>
                <TouchableOpacity
                    onPress={() => setScanData(null)}
                    style={styles.button}
                >
                    <Text style={styles.buttonText}>Scan again</Text>
                </TouchableOpacity>
                <TouchableOpacity
                    onPress={() => navigation.navigate("Home")}
                    style={[styles.button, styles.buttonOutline]}
                >
                    <Text style={styles.buttonOutlineText}>Return to home</Text>
                </TouchableOpacity>
            </View>
        </View>
        )
    }

    return (
        <View style={styles.container}>
            <BarCodeScanner 
                style={StyleSheet.absoluteFillObject}
                onBarCodeScanned={scanData ? undefined : handleBarCodeScanned}
            />
        </View>
      );
    }

export default QRScannerScreen

const styles = StyleSheet.create({
    container: {
      flex: 1,
      backgroundColor: '#fff',
      alignItems: 'center',
      justifyContent: 'center',
    },
    headerText: {
        color: '#0782F9',
        fontWeight: '700',
        fontSize: 20,
        marginTop: 40
    },
    deviceText: {
        color: 'white',
        fontSize: 20,
    },
    deviceTextContainer: {
        backgroundColor: 'black',
        width: '50%',
        height: '5%',
        borderRadius: 10,
        marginTop: 10,
        alignItems: 'center',
        justifyContent: 'center',
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
  });