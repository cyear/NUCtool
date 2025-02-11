const { listen } = window.__TAURI__.event;

// 左风扇曲线
const leftFanCtx = document.getElementById('leftFanCurve').getContext('2d');
const leftFanCurve = new Chart(leftFanCtx, {
    type: 'line',
    data: {
        labels: Array.from({length: 15}, (_, i) => (i + 30) + Math.round(i * 4)),  // 30 - 95度
        datasets: [{
            label: 'CPU风扇速度',
            data: Array(66).fill(50),  // 默认风扇速度
            borderColor: 'blue',
            fill: false
        }]
    },
    options: {
        scales: {
            x: {title: {display: true, text: '温度 (°C)'}},
            y: {title: {display: true, text: '风扇速度 (%)'}, min: 0, max: 100}
        },
        plugins: {
            dragData: {
                round: 0,
                onDrag: function (e, datasetIndex, index, value) {
                    console.log(`Left Fan - Temperature: ${leftFanCurve.data.labels[index]}, Speed: ${value}%`);
                }
            },
        },
        animations: {
            tension: {
                duration: 1000,
                easing: 'linear',
                from: 1,
                to: 0,
                loop: false
            }
        },
        cubicInterpolationMode: 'monotone'
    }
});

// 右风扇曲线
const rightFanCtx = document.getElementById('rightFanCurve').getContext('2d');
const rightFanCurve = new Chart(rightFanCtx, {
    type: 'line',
    data: {
        labels: Array.from({length: 15}, (_, i) => (i + 30) + Math.round(i * 4)),
        datasets: [{
            label: 'GPU风扇速度',
            data: Array(66).fill(50),
            borderColor: 'green',
            fill: false
        }]
    },
    options: {
        scales: {
            x: {title: {display: true, text: '温度 (°C)'}},
            y: {title: {display: true, text: '风扇速度 (%)'}, min: 0, max: 100}
        },
        plugins: {
            dragData: {
                round: 0,
                onDrag: function (e, datasetIndex, index, value) {
                    console.log(`Right Fan - Temperature: ${rightFanCurve.data.labels[index]}, Speed: ${value}%`);
                }
            }
        },
        animations: {
            tension: {
                duration: 1000,
                easing: 'easeInBounce',
                from: 1,
                to: 0,
                loop: false
            }
        },
        cubicInterpolationMode: 'monotone'
    }
});

const leftFanSpeedCtx = document.getElementById('left_fan_speed').getContext('2d');
const rightFanSpeedCtx = document.getElementById('right_fan_speed').getContext('2d');

// 初始化风扇实时转速图表
function initSpeedCharts() {
    const leftFanSpeedChart = new Chart(leftFanSpeedCtx, {
        type: 'line',
        data: {
            labels: Array.from({length: 21}, (_, i) => i * 3),
            datasets: [{
                label: 'CPU风扇实时转速',
                data: Array(21).fill(0),
                borderColor: 'blue',
                fill: false
            }, {
                label: 'GPU风扇实时转速',
                data: Array(21).fill(0),
                borderColor: 'green',
                fill: false
            }]
        },
        options: {
            scales: {
                x: {title: {display: true, text: '时间 (秒)'}},
                y: {title: {display: true, text: '转速 (RPM)'}, min: 0, max: 6000}
            },
            pointStyle: false,
            cubicInterpolationMode: 'monotone'
        },
    });

    const rightFanSpeedChart = new Chart(rightFanSpeedCtx, {
        type: 'line',
        data: {
            labels: Array.from({length: 21}, (_, i) => i * 3),
            datasets: [{
                label: 'CPU实时温度',
                data: Array(21).fill(0),
                borderColor: 'blue',
                fill: false
            }, {
                label: 'GPU实时温度',
                data: Array(21).fill(0),
                borderColor: 'green',
                fill: false
            }]
        },
        options: {
            scales: {
                x: {title: {display: true, text: '时间 (秒)'}},
                y: {title: {display: true, text: '温度 (℃)'}, min: 0, max: 100}
            },
            pointStyle: false,
            cubicInterpolationMode: 'monotone'
        }
    });

    return {leftFanSpeedChart, rightFanSpeedChart};
}

// 更新风扇实时转速数据
function updateFanSpeeds(leftFanSpeedChart, rightFanSpeedChart, left_fan_speed, right_fan_speed, left_temp, right_temp) {
    if (left_fan_speed < 0 || right_fan_speed < 0 || left_fan_speed > 7000 || right_fan_speed > 7000 || left_temp < 0 || right_temp < 0 || left_temp > 100 || right_temp > 100) {
        return;
    }
    leftFanSpeedChart.data.datasets[0].data.push(left_fan_speed);
    leftFanSpeedChart.data.datasets[0].data.shift(); // 移除最早的数据
    leftFanSpeedChart.data.datasets[1].data.push(right_fan_speed);
    leftFanSpeedChart.data.datasets[1].data.shift();
    rightFanSpeedChart.data.datasets[0].data.push(left_temp);
    rightFanSpeedChart.data.datasets[0].data.shift();
    rightFanSpeedChart.data.datasets[1].data.push(right_temp);
    rightFanSpeedChart.data.datasets[1].data.shift();
    // console.log(`Left Fan - Speed: ${leftFanSpeedChart.data.datasets[0].data}, Right Fan - Speed: ${rightFanSpeedChart.data.datasets[0].data}`);
    leftFanSpeedChart.update();
    rightFanSpeedChart.update();
}

async function loadConfigData() {
    const fanData = await window.__TAURI__.core.invoke('load_fan_config');
    if (fanData) {
        // 更新左风扇曲线数据
        leftFanCurve.data.datasets[0].data = fanData.left_fan.map(point => point.speed);
        leftFanCurve.update();

        // 更新右风扇曲线数据
        rightFanCurve.data.datasets[0].data = fanData.right_fan.map(point => point.speed);
        rightFanCurve.update();
    } else {
        console.log('未找到配置文件或读取失败');
    }
}

document.addEventListener('DOMContentLoaded', async () => {
    // const { leftFanCurve, rightFanCurve } = await initFanCurves();
    const {leftFanSpeedChart, rightFanSpeedChart} = initSpeedCharts();

    const startStopButton = document.getElementById('startStopButton');
    const saveConfigButton = document.getElementById('saveConfigButton');
    let isRunning = false;
    // await loadConfigData();
    // 定时更新风扇转速

    // setInterval(async () => {
    //     // const speeds = await window.__TAURI__.core.invoke('get_fan_speeds');
    //     // updateFanSpeeds(leftFanSpeedChart, rightFanSpeedChart, speeds.left_fan_speed, speeds.right_fan_speed, speeds.left_temp, speeds.right_temp);
    // }, 2500);
    async function listen_to_greet() {
      await listen('get-fan-speeds', (speeds) => {
        // event.payload 才是实际的结构体
        console.log(speeds.payload);
        updateFanSpeeds(leftFanSpeedChart, rightFanSpeedChart, speeds.payload.left_fan_speed, speeds.payload.right_fan_speed, speeds.payload.left_temp, speeds.payload.right_temp);
      });
    }
    await window.__TAURI__.core.invoke('get_fan_speeds');
    await listen_to_greet();
    // 按钮点击事件
    startStopButton.addEventListener('click', () => {
        isRunning = !isRunning;
        if (isRunning) {
            startStopButton.querySelector('a').textContent = 'Stop';
            startStopButton.style.backgroundColor = 'rgb(255, 0, 0, 0.3)';
            startStopButton.classList.remove('stopped');

            // 获取数据并传递给 Rust
            const fanData = getFanCurveData();
            window.__TAURI__.core.invoke('start_fan_control', {fanData});
        } else {
            startStopButton.querySelector('a').textContent = 'Start';
            startStopButton.style.backgroundColor = 'rgb(255, 182, 193, 0.3)';
            startStopButton.classList.add('stopped');

            // 停止风扇控制
            window.__TAURI__.core.invoke('stop_fan_control');
        }
    });
    // 加载配置按钮
    const loadConfigButton = document.getElementById('loadConfigButton');
    loadConfigButton.addEventListener('click', async () => {
        await loadConfigData();
    });
    // 保存配置按钮
    saveConfigButton.addEventListener('click', () => {
        const fanData = getFanCurveData();
        window.__TAURI__.core.invoke('save_fan_config', {fanData});
    });
});

// 获取所有点信息并传递给 Rust
function getFanCurveData() {
    const leftFanData = leftFanCurve.data.labels.map((temp, index) => {
        return {temperature: temp, speed: leftFanCurve.data.datasets[0].data[index]};
    });

    const rightFanData = rightFanCurve.data.labels.map((temp, index) => {
        return {temperature: temp, speed: rightFanCurve.data.datasets[0].data[index]};
    });

    return {left_fan: leftFanData, right_fan: rightFanData};
}
