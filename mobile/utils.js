// Borrowed from https://stackoverflow.com/questions/48384163/javascript-remove-day-name-from-date

export function formatDateString(myDate) {
    var date = new Date(myDate);
    date.setDate(date.getDate() + 1);
    date = date.toDateString().split(' ').slice(1).join(' ');
    date = date.slice(0,6) + ',' + date.slice(6);
    return date
}

export function formatTimestampDate(timestamp) {
    const date = timestamp.toDate();
    return date.toISOString().split('T')[0];
}

export function getTimeTimestamp(timestamp) {
    const date = timestamp.toDate();
    const amPm = date.getHours() >= 12 ? 'PM' : 'AM';
    const hour = date.getHours() > 12 ? date.getHours() - 12 : date.getHours();
    const hourString = hour < 10 ? `0${hour}` : `${hour}`;
    const minuteString = date.getMinutes() < 10 ? `0${date.getMinutes()}` : `${date.getMinutes()}`; 
    return `${hourString}:${minuteString} ${amPm}`;
}

export function capitalizeString(string) {
    return string[0].toUpperCase() + string.slice(1);
}

export function calculateSquatFeedback(reps) {
  if (!reps || reps.length == 0) return '';
  var classArray = [];
  reps.forEach((rep) => {
    classArray.push(rep.class);
  })
  var mostCommonClass = mode(classArray);
  return `Most common correction: ${mostCommonClass}`
}

export function calculatePushupFeedback(reps) {
  if (!reps || reps.length == 0) return '';
  var classArray = [];
  reps.forEach((rep) => {
    rep.class.split(', ').map((className) => {
      classArray.push(className);
    })
  })
  var mostCommonClass = mode(classArray);
  return `Most common correction: ${mostCommonClass}`
}

// Taken from: https://stackoverflow.com/questions/1053843/get-the-element-with-the-highest-occurrence-in-an-array
function mode(array)
{
    if(array.length == 0)
        return null;
    var modeMap = {};
    var maxEl = array[0], maxCount = 1;
    for(var i = 0; i < array.length; i++)
    {
        var el = array[i];
        if(modeMap[el] == null)
            modeMap[el] = 1;
        else
            modeMap[el]++;  
        if(modeMap[el] > maxCount)
        {
            maxEl = el;
            maxCount = modeMap[el];
        }
    }
    return maxEl;
}

export function calculateAccuracyPercentage(reps) {
    if (!reps || reps.length == 0) return '';
    console.log(reps);
    const repCount = reps.length;
    const acceptableCount = reps.filter((rep) => {
        return rep.class === 'Acceptable';
    }).length;
    return `${Math.round(acceptableCount/repCount * 100)}% acceptable`;
}

export function getSquatClassColor(className) {
    switch(className) {
      case("Acceptable"):
        return '#249225';
      case("Anterior Knee"):
        return "#FFBF00";
      case("Bent Over"):
        return "#FF7FF0";
      case("Half Squat"):
        return "#FF0000";
      case("Knee Valgus"):
        return "#4B0082";
      case("Knee Varus"):
        return "#0000FF";
      case("Other"):
        return "9400D3";
      default:
        return "#000000"
    }
}

export function getPushupClassColor(className) {
  switch(className) {
    case("Acceptable"):
      return '#249225';
    case("Half Push-Up"):
      return "#FFBF00";
    case("Titled Neck"):
      return "#FF7FF0";
    case("Pelvis Curved"):
      return "#FF0000";
    case("Bent Knee"):
      return "#4B0082";
    case("Other"):
      return "9400D3";
    default:
      return "#000000"
  }
}