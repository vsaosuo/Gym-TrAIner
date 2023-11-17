// Import the functions you need from the SDKs you need
import * as firebase from "firebase";
// TODO: Add SDKs for Firebase products that you want to use
// https://firebase.google.com/docs/web/setup#available-libraries

// Your web app's Firebase configuration
// For Firebase JS SDK v7.20.0 and later, measurementId is optional
const firebaseConfig = {
    apiKey: "AIzaSyCdBN3IzMwxpibAGKNNxE0PtGyfufZFCcA",
    authDomain: "gym-tr-ai-ner.firebaseapp.com",
    projectId: "gym-tr-ai-ner",
    storageBucket: "gym-tr-ai-ner.appspot.com",
    messagingSenderId: "290465688260",
    appId: "1:290465688260:web:8e14e2ff92975519493791"
  };
  
  // Initialize Firebase
  let app;
  if (firebase.apps.length === 0) {
      app = firebase.initializeApp(firebaseConfig);
  } else {
      app = firebase.app();
  }
  const auth = firebase.auth();
  const database = firebase.firestore();
  const storage = firebase.storage();
  
  export { auth, database, storage, firebase }