#!/usr/bin/env python3
"""
ConHub Database Management Script

Handles database operations like migrations, user management, and health checks.
"""

import os
import sys
import argparse
import psycopg2
from psycopg2.extras import RealDictCursor
from dotenv import load_dotenv
import json

# Load environment variables
load_dotenv()

class DatabaseManager:
    """Manages database operations for ConHub."""
    
    def __init__(self):
        self.connection_string = (
            os.getenv("DATABASE_URL_NEON") or 
            os.getenv("DATABASE_URL") or
            "postgresql://neondb_owner:npg_w8jLMEkgsxc9@ep-wispy-credit-aazkw4fu-pooler.westus3.azure.neon.tech/neondb?sslmode=require&channel_binding=require"
        )
    
    def get_connection(self):
        """Get database connection."""
        try:
            conn = psycopg2.connect(self.connection_string)
            return conn
        except Exception as e:
            print(f"❌ Failed to connect to database: {e}")
            sys.exit(1)
    
    def health_check(self):
        """Check database health and connectivity."""
        print("🔌 Checking database connection...")
        
        try:
            with self.get_connection() as conn:
                with conn.cursor() as cur:
                    cur.execute("SELECT 1")
                    result = cur.fetchone()
                    
                    if result:
                        print("✅ Database connection successful!")
                        
                        # Check auth tables
                        auth_tables = ['users', 'user_sessions', 'rate_limits', 'security_audit_log']
                        print("🔍 Checking auth tables:")
                        
                        for table in auth_tables:
                            cur.execute(
                                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = %s",
                                (table,)
                            )
                            exists = cur.fetchone()[0] > 0
                            status = "✅" if exists else "❌"
                            print(f"  {status} {table}")
                        
                        # Check user count
                        try:
                            cur.execute("SELECT COUNT(*) FROM users")
                            user_count = cur.fetchone()[0]
                            print(f"📊 Total users: {user_count}")
                        except Exception as e:
                            print(f"⚠️  Could not count users: {e}")
                    
        except Exception as e:
            print(f"❌ Database health check failed: {e}")
            sys.exit(1)
    
    def delete_user(self, email):
        """Delete a user by email."""
        print(f"🗑️  Deleting user: {email}")
        
        try:
            with self.get_connection() as conn:
                with conn.cursor() as cur:
                    cur.execute("DELETE FROM users WHERE email = %s", (email,))
                    rows_affected = cur.rowcount
                    
                    if rows_affected > 0:
                        print(f"✅ User deleted successfully! Rows affected: {rows_affected}")
                    else:
                        print(f"ℹ️  No user found with email: {email}")
                        
        except Exception as e:
            print(f"❌ Failed to delete user: {e}")
            sys.exit(1)
    
    def list_users(self):
        """List all users in the database."""
        print("📋 Listing all users:")
        
        try:
            with self.get_connection() as conn:
                with conn.cursor(cursor_factory=RealDictCursor) as cur:
                    cur.execute(
                        "SELECT id, email, name, organization, role, is_active, created_at FROM users ORDER BY created_at DESC"
                    )
                    users = cur.fetchall()
                    
                    if users:
                        for user in users:
                            status = "🟢" if user['is_active'] else "🔴"
                            print(f"  {status} {user['email']} ({user['name']}) - {user['role']} - {user['created_at']}")
                    else:
                        print("  No users found")
                        
        except Exception as e:
            print(f"❌ Failed to list users: {e}")
            sys.exit(1)
    
    def run_migration(self, migration_file):
        """Run a specific migration file."""
        print(f"🔄 Running migration: {migration_file}")
        
        if not os.path.exists(migration_file):
            print(f"❌ Migration file not found: {migration_file}")
            sys.exit(1)
        
        try:
            with open(migration_file, 'r') as f:
                migration_sql = f.read()
            
            with self.get_connection() as conn:
                with conn.cursor() as cur:
                    cur.execute(migration_sql)
                    print("✅ Migration completed successfully!")
                    
        except Exception as e:
            print(f"❌ Migration failed: {e}")
            sys.exit(1)


def main():
    """Main CLI interface."""
    parser = argparse.ArgumentParser(description="ConHub Database Management")
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Health check command
    subparsers.add_parser('health', help='Check database health')
    
    # Delete user command
    delete_parser = subparsers.add_parser('delete-user', help='Delete a user by email')
    delete_parser.add_argument('email', help='Email of the user to delete')
    
    # List users command
    subparsers.add_parser('list-users', help='List all users')
    
    # Migration command
    migrate_parser = subparsers.add_parser('migrate', help='Run a migration file')
    migrate_parser.add_argument('file', help='Path to migration file')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        sys.exit(1)
    
    db_manager = DatabaseManager()
    
    if args.command == 'health':
        db_manager.health_check()
    elif args.command == 'delete-user':
        db_manager.delete_user(args.email)
    elif args.command == 'list-users':
        db_manager.list_users()
    elif args.command == 'migrate':
        db_manager.run_migration(args.file)


if __name__ == "__main__":
    main()
