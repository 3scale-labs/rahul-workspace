const express = require("express")
const app = express()
const PORT = 3001

function sleep(ms) {
    return new Promise((resolve) => {
      setTimeout(resolve, ms);
    });
} 

app.all("*", async (req, res, next) => {
    await sleep(2000)
    console.log(`Received request: ${req}`)
    res.send('MEOWW')
})

app.listen(PORT,()=>{console.log(`Lstening on port ${PORT}`)})