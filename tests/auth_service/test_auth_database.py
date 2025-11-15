"""
Auth Service Database Tests

Tests for database operations in the auth service.
"""

import pytest
import psycopg2
import os
from dotenv import load_dotenv

load_dotenv()

class TestAuthDatabase:
    """Test suite for auth service database operations."""
    
    @pytest.fixture(scope="class")
    def db_connection(self):
        """Get database connection for testing."""
        connection_string = (
            os.getenv("DATABASE_URL_NEON") or 
            os.getenv("DATABASE_URL") or
            "postgresql://neondb_owner:npg_w8jLMEkgsxc9@ep-wispy-credit-aazkw4fu-pooler.westus3.azure.neon.tech/neondb?sslmode=require&channel_binding=require"
        )
        
        conn = psycopg2.connect(connection_string)
        yield conn
        conn.close()
    
    def test_database_connection(self, db_connection):
        """Test that database connection is working."""
        cursor = db_connection.cursor()
        cursor.execute("SELECT 1")
        result = cursor.fetchone()
        assert result[0] == 1
        cursor.close()
    
    def test_users_table_exists(self, db_connection):
        """Test that users table exists and has correct structure."""
        cursor = db_connection.cursor()
        
        # Check if table exists
        cursor.execute("""
            SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_name = 'users'
            )
        """)
        assert cursor.fetchone()[0] is True
        
        # Check table structure
        cursor.execute("""
            SELECT column_name, data_type 
            FROM information_schema.columns 
            WHERE table_name = 'users'
            ORDER BY ordinal_position
        """)
        
        columns = cursor.fetchall()
        column_names = [col[0] for col in columns]
        
        required_columns = [
            'id', 'email', 'password_hash', 'name', 'role', 
            'is_active', 'created_at', 'updated_at'
        ]
        
        for required_col in required_columns:
            assert required_col in column_names, f"Missing column: {required_col}"
        
        cursor.close()
    
    def test_user_roles_enum(self, db_connection):
        """Test that user_role enum exists with correct values."""
        cursor = db_connection.cursor()
        
        cursor.execute("""
            SELECT enumlabel 
            FROM pg_enum e 
            JOIN pg_type t ON e.enumtypid = t.oid 
            WHERE t.typname = 'user_role'
        """)
        
        roles = [row[0] for row in cursor.fetchall()]
        expected_roles = ['admin', 'user', 'moderator']
        
        for role in expected_roles:
            assert role in roles, f"Missing role: {role}"
        
        cursor.close()
    
    def test_rate_limits_table(self, db_connection):
        """Test that rate_limits table exists for auth service."""
        cursor = db_connection.cursor()
        
        cursor.execute("""
            SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_name = 'rate_limits'
            )
        """)
        assert cursor.fetchone()[0] is True
        cursor.close()
    
    def test_user_sessions_table(self, db_connection):
        """Test that user_sessions table exists for JWT management."""
        cursor = db_connection.cursor()
        
        cursor.execute("""
            SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_name = 'user_sessions'
            )
        """)
        assert cursor.fetchone()[0] is True
        cursor.close()
