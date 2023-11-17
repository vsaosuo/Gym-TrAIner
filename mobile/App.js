import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import HomeScreen from './screens/HomeScreen';
import LoginScreen from './screens/LoginScreen';
import QRScannerScreen from './screens/QRScannerScreen';
import RegisterScreen from './screens/RegisterScreen';
import WorkoutScreen from './screens/WorkoutScreen';
import WorkoutSummaryScreen from './screens/WorkoutSummaryScreen';


const Stack = createNativeStackNavigator();

export default function App() {
  return (
    <NavigationContainer>
      <Stack.Navigator>
        <Stack.Screen name="Login" options ={{ headerShown: false }} component={LoginScreen}/>
        <Stack.Screen name="Register" options ={{ headerShown: false }} component={RegisterScreen}/>
        <Stack.Screen name="Home" options ={{ headerShown: false }} component={HomeScreen}/>
        <Stack.Screen name="QRScanner" options ={{ headerShown: false }} component={QRScannerScreen}/>
        <Stack.Screen name="WorkoutSummary" options = {{ headerShown: false }} component={WorkoutSummaryScreen}/>
        <Stack.Screen name="Workout" options = {{ headerShown: false }} component = {WorkoutScreen}/>
      </Stack.Navigator>
    </NavigationContainer>
  );
}