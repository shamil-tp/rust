const express = require('express');
const bcrypt = require('bcryptjs')
const connectDB = require('./db');
const User = require('./User'); 

const app = express();
app.use(express.json());
app.use(express.static('public'))
// Initialize database connection
connectDB();

// GET endpoint to fetch user by credentials
//const bcrypt = require('bcryptjs'); // Make sure this is imported at the top of your file

app.get('/api/user', async (req, res) => {
  try {
    const { username, password } = req.query;

    if (!username || !password) {
      return res.status(400).json({ error: 'Username and password are required' });
    }

    // 1. Find the user by username ONLY and explicitly include the hidden password field
    const user = await User.findOne({ username }).select('+password');

    // 2. Check if the user exists
    if (!user) {
      return res.status(401).json({ error: 'Invalid credentials or user not found' });
    }

    // 3. Compare the plain text password from the query with the hashed password in the DB
    const isMatch = await bcrypt.compare(password, user.password);

    if (!isMatch) {
      return res.status(401).json({ error: 'Invalid credentials or user not found' });
    }

    // 4. Strip the password from the object before sending it to the client
    const userResponse = user.toObject();
    delete userResponse.password;

    res.json(userResponse);
  } catch (error) {
    res.status(500).json({ error: 'Internal server error', details: error.message });
  }
});
app.post('/api/users', async (req, res) => {
  try {
    const { username, email, password, firstName, lastName, bio } = req.body;

    // 1. Check if a user with the same email or username already exists
    const existingUser = await User.findOne({
      $or: [{ email: email?.toLowerCase() }, { username }]
    });

    if (existingUser) {
      return res.status(400).json({ 
        error: 'A user with this username or email already exists' 
      });
    }

    // 2. Hash the password before saving
    const salt = await bcrypt.genSalt(10);
    const hashedPassword = await bcrypt.hash(password, salt);

    // 3. Create and save the new user
    const newUser = await User.create({
      username,
      email,
      password: hashedPassword,
      firstName,
      lastName,
      bio
    });

    // 4. Remove the password from the response object
    // Note: Even though we set select: false in the schema, User.create() 
    // returns the full document in memory, so we must delete it manually here.
    const userResponse = newUser.toObject();
    delete userResponse.password;

    // 5. Send success response (201 Created)
    res.status(201).json({
      message: 'User created successfully',
      user: userResponse
    });

  } catch (error) {
    // Handle specific Mongoose validation errors (e.g., missing fields, string too short)
    if (error.name === 'ValidationError') {
      const messages = Object.values(error.errors).map(val => val.message);
      return res.status(400).json({ error: messages });
    }

    // Handle unexpected server errors
    console.error('Error creating user:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});
app.listen(3000, () => console.log('Server running on port 3000'));