const mongoose = require('mongoose');

const connectDB = async () => {
  try {
    // Replace the URI with your MongoDB connection string or use an environment variable
    const conn = await mongoose.connect("please paste uri" || 'mongodb://127.0.0.1:27017/my_database');
    console.log(`MongoDB Connected: ${conn.connection.host}`);
  } catch (error) {
    console.error(`Database Connection Error: ${error.message}`);
    process.exit(1); 
  }
};

module.exports = connectDB;

