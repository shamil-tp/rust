const mongoose = require('mongoose');

const userSchema = new mongoose.Schema(
  {
    // Authentication & Identification
    username: { 
      type: String, 
      required: [true, 'Username is required'], 
      unique: true,
      trim: true,
      minlength: [3, 'Username must be at least 3 characters long'],
      maxlength: [30, 'Username cannot exceed 30 characters']
    },
    email: { 
      type: String, 
      required: [true, 'Email is required'], 
      unique: true,
      trim: true,
      lowercase: true,
      match: [/^\w+([\.-]?\w+)*@\w+([\.-]?\w+)*(\.\w{2,3})+$/, 'Please fill a valid email address']
    },
    password: { 
      type: String, 
      required: [true, 'Password is required'],
      minlength: [6, 'Password must be at least 6 characters long'],
      select: false // Automatically excludes password from query results unless explicitly requested
    },

    // Profile Information
    firstName: { 
      type: String, 
      trim: true,
      maxlength: [50, 'First name cannot exceed 50 characters']
    },
    lastName: { 
      type: String, 
      trim: true,
      maxlength: [50, 'Last name cannot exceed 50 characters']
    },
    profilePicture: { 
      type: String, 
      default: 'default-avatar.png' // Can be a URL to an image hosting service like Cloudinary or AWS S3
    },
    bio: {
      type: String,
      maxlength: [500, 'Bio cannot exceed 500 characters']
    },

    // Authorization & Roles
    role: { 
      type: String, 
      enum: ['user', 'moderator', 'admin'], 
      default: 'user' 
    },

    // Account Status & Verification
    isActive: { 
      type: Boolean, 
      default: true 
    },
    isEmailVerified: { 
      type: Boolean, 
      default: false 
    },

    // Security & Account Recovery
    lastLogin: { 
      type: Date,
      default: null
    },
    resetPasswordToken: {
      type: String
    },
    resetPasswordExpire: {
      type: Date
    }
  }, 
  {
    // Mongoose configuration options
    timestamps: true // Automatically creates 'createdAt' and 'updatedAt' fields
  }
);

// Optional: Virtual field for full name (does not get saved in DB, just calculated on the fly)
userSchema.virtual('fullName').get(function() {
  if (this.firstName && this.lastName) {
    return `${this.firstName} ${this.lastName}`;
  }
  return this.firstName || this.lastName || this.username;
});

// Ensure virtual fields are serialized when converting document to JSON
userSchema.set('toJSON', { virtuals: true });
userSchema.set('toObject', { virtuals: true });

module.exports = mongoose.model('User', userSchema);