#!/usr/bin/env python3
"""
Migration script to transition from MCP services to the new plugin system.
This script helps migrate configurations, data, and settings from the old
microservices architecture to the unified plugin-based system.
"""

import json
import os
import sys
import shutil
import argparse
import logging
from pathlib import Path
from typing import Dict, List, Any, Optional
import requests
import time

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class PluginMigrator:
    def __init__(self, config_path: str = None):
        self.config_path = config_path or "migration_config.json"
        self.backup_dir = Path("migration_backup")
        self.plugins_api_base = "http://localhost:3020"
        self.old_services = {
            "mcp-google-drive": {
                "plugin_type": "source",
                "plugin_name": "google-drive",
                "config_mapping": {
                    "client_id": "client_id",
                    "client_secret": "client_secret",
                    "refresh_token": "refresh_token",
                    "folder_id": "folder_id"
                }
            },
            "mcp-dropbox": {
                "plugin_type": "source", 
                "plugin_name": "dropbox",
                "config_mapping": {
                    "access_token": "access_token",
                    "app_key": "app_key",
                    "app_secret": "app_secret",
                    "folder_path": "folder_path"
                }
            },
            "mcp-github-copilot": {
                "plugin_type": "agent",
                "plugin_name": "github-copilot",
                "config_mapping": {
                    "api_key": "api_key",
                    "model": "model",
                    "max_tokens": "max_tokens"
                }
            },
            "mcp-cline": {
                "plugin_type": "agent",
                "plugin_name": "cline",
                "config_mapping": {
                    "api_key": "api_key",
                    "model": "model",
                    "base_url": "base_url",
                    "max_tokens": "max_tokens",
                    "temperature": "temperature"
                }
            },
            "mcp-amazon-q": {
                "plugin_type": "agent",
                "plugin_name": "amazon-q",
                "config_mapping": {
                    "aws_access_key_id": "aws_access_key_id",
                    "aws_secret_access_key": "aws_secret_access_key",
                    "aws_region": "aws_region",
                    "model_id": "model_id"
                }
            }
        }
        
    def load_migration_config(self) -> Dict[str, Any]:
        """Load migration configuration from file."""
        if not os.path.exists(self.config_path):
            logger.info(f"Creating default migration config at {self.config_path}")
            default_config = {
                "docker_compose_path": "docker-compose.yml",
                "env_files": [".env", ".env.local"],
                "backup_enabled": True,
                "dry_run": False,
                "plugins_service_url": self.plugins_api_base,
                "migration_steps": {
                    "backup_configs": True,
                    "extract_env_vars": True,
                    "create_plugin_configs": True,
                    "migrate_data": True,
                    "validate_plugins": True,
                    "cleanup_old_services": False
                }
            }
            with open(self.config_path, 'w') as f:
                json.dump(default_config, f, indent=2)
            return default_config
        
        with open(self.config_path, 'r') as f:
            return json.load(f)
    
    def create_backup(self) -> None:
        """Create backup of current configuration."""
        logger.info("Creating backup of current configuration...")
        
        if self.backup_dir.exists():
            shutil.rmtree(self.backup_dir)
        self.backup_dir.mkdir(parents=True)
        
        # Backup docker-compose.yml
        if os.path.exists("docker-compose.yml"):
            shutil.copy2("docker-compose.yml", self.backup_dir / "docker-compose.yml.backup")
        
        # Backup environment files
        for env_file in [".env", ".env.local", ".env.production"]:
            if os.path.exists(env_file):
                shutil.copy2(env_file, self.backup_dir / f"{env_file}.backup")
        
        # Backup existing plugin configs
        plugins_config_dir = Path("plugins/config")
        if plugins_config_dir.exists():
            shutil.copytree(plugins_config_dir, self.backup_dir / "plugins_config_backup")
        
        logger.info(f"Backup created in {self.backup_dir}")
    
    def extract_service_configs(self) -> Dict[str, Dict[str, Any]]:
        """Extract configuration from docker-compose.yml and environment files."""
        logger.info("Extracting service configurations...")
        
        configs = {}
        
        # Load environment variables
        env_vars = {}
        for env_file in [".env", ".env.local"]:
            if os.path.exists(env_file):
                with open(env_file, 'r') as f:
                    for line in f:
                        line = line.strip()
                        if line and not line.startswith('#') and '=' in line:
                            key, value = line.split('=', 1)
                            env_vars[key.strip()] = value.strip().strip('"\'')
        
        # Extract configurations for each old service
        for service_name, service_info in self.old_services.items():
            service_config = {}
            
            # Map environment variables to plugin configuration
            for old_key, new_key in service_info["config_mapping"].items():
                # Try different environment variable naming patterns
                env_key_patterns = [
                    f"{service_name.upper().replace('-', '_')}_{old_key.upper()}",
                    f"{service_info['plugin_name'].upper().replace('-', '_')}_{old_key.upper()}",
                    old_key.upper(),
                    f"MCP_{old_key.upper()}"
                ]
                
                for pattern in env_key_patterns:
                    if pattern in env_vars:
                        service_config[new_key] = env_vars[pattern]
                        break
            
            if service_config:
                configs[service_name] = {
                    "plugin_type": service_info["plugin_type"],
                    "plugin_name": service_info["plugin_name"],
                    "config": service_config
                }
        
        logger.info(f"Extracted configurations for {len(configs)} services")
        return configs
    
    def create_plugin_configs(self, service_configs: Dict[str, Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Create plugin configuration files from extracted service configs."""
        logger.info("Creating plugin configurations...")
        
        plugin_configs = []
        
        for service_name, service_data in service_configs.items():
            instance_id = f"{service_data['plugin_name']}-default"
            
            plugin_config = {
                "instance_id": instance_id,
                "plugin_type": service_data["plugin_type"],
                "plugin_name": service_data["plugin_name"],
                "enabled": True,
                "auto_start": True,
                "config": service_data["config"]
            }
            
            plugin_configs.append(plugin_config)
            logger.info(f"Created config for {service_data['plugin_name']} plugin")
        
        return plugin_configs
    
    def save_plugin_configs(self, plugin_configs: List[Dict[str, Any]], dry_run: bool = False) -> None:
        """Save plugin configurations to the plugins service."""
        logger.info("Saving plugin configurations...")
        
        # Ensure plugins config directory exists
        config_dir = Path("plugins/config")
        if not dry_run:
            config_dir.mkdir(parents=True, exist_ok=True)
        
        # Save to plugins.json file
        plugins_json_path = config_dir / "plugins.json"
        
        if dry_run:
            logger.info(f"[DRY RUN] Would save {len(plugin_configs)} plugin configs to {plugins_json_path}")
            for config in plugin_configs:
                logger.info(f"[DRY RUN] Plugin config: {config['instance_id']}")
        else:
            # Load existing configs if any
            existing_configs = []
            if plugins_json_path.exists():
                with open(plugins_json_path, 'r') as f:
                    data = json.load(f)
                    existing_configs = data.get('plugins', [])
            
            # Merge with new configs (avoid duplicates)
            existing_ids = {config['instance_id'] for config in existing_configs}
            for config in plugin_configs:
                if config['instance_id'] not in existing_ids:
                    existing_configs.append(config)
            
            # Save updated configs
            with open(plugins_json_path, 'w') as f:
                json.dump({"plugins": existing_configs}, f, indent=2)
            
            logger.info(f"Saved {len(plugin_configs)} plugin configurations to {plugins_json_path}")
    
    def validate_plugins_service(self) -> bool:
        """Validate that the plugins service is running and accessible."""
        logger.info("Validating plugins service...")
        
        try:
            response = requests.get(f"{self.plugins_api_base}/health", timeout=10)
            if response.status_code == 200:
                logger.info("Plugins service is running and accessible")
                return True
            else:
                logger.error(f"Plugins service returned status {response.status_code}")
                return False
        except requests.exceptions.RequestException as e:
            logger.error(f"Failed to connect to plugins service: {e}")
            return False
    
    def load_plugin_configs_via_api(self, plugin_configs: List[Dict[str, Any]], dry_run: bool = False) -> bool:
        """Load plugin configurations via the plugins API."""
        logger.info("Loading plugin configurations via API...")
        
        if dry_run:
            logger.info(f"[DRY RUN] Would load {len(plugin_configs)} configs via API")
            return True
        
        success_count = 0
        
        for config in plugin_configs:
            try:
                # Create plugin configuration
                response = requests.post(
                    f"{self.plugins_api_base}/api/config",
                    json=config,
                    timeout=30
                )
                
                if response.status_code in [200, 201]:
                    logger.info(f"Successfully created config for {config['instance_id']}")
                    success_count += 1
                    
                    # Try to start the plugin
                    start_response = requests.post(
                        f"{self.plugins_api_base}/api/lifecycle/{config['instance_id']}/start",
                        timeout=30
                    )
                    
                    if start_response.status_code == 200:
                        logger.info(f"Successfully started plugin {config['instance_id']}")
                    else:
                        logger.warning(f"Failed to start plugin {config['instance_id']}: {start_response.text}")
                        
                else:
                    logger.error(f"Failed to create config for {config['instance_id']}: {response.text}")
                    
            except requests.exceptions.RequestException as e:
                logger.error(f"API request failed for {config['instance_id']}: {e}")
        
        logger.info(f"Successfully loaded {success_count}/{len(plugin_configs)} plugin configurations")
        return success_count == len(plugin_configs)
    
    def validate_migration(self) -> bool:
        """Validate that the migration was successful."""
        logger.info("Validating migration...")
        
        try:
            # Check plugins status
            response = requests.get(f"{self.plugins_api_base}/api/status", timeout=10)
            if response.status_code != 200:
                logger.error("Failed to get plugins status")
                return False
            
            status = response.json()
            logger.info(f"Plugins status: {status}")
            
            # Check that we have active plugins
            if status.get('total_active', 0) == 0:
                logger.warning("No active plugins found after migration")
                return False
            
            logger.info("Migration validation successful")
            return True
            
        except requests.exceptions.RequestException as e:
            logger.error(f"Validation failed: {e}")
            return False
    
    def cleanup_old_services(self, dry_run: bool = False) -> None:
        """Clean up old service configurations (optional)."""
        logger.info("Cleaning up old service configurations...")
        
        if dry_run:
            logger.info("[DRY RUN] Would remove old service configurations from docker-compose.yml")
            return
        
        # This is a placeholder - in practice, you might want to:
        # 1. Comment out old services in docker-compose.yml
        # 2. Remove old environment variables
        # 3. Archive old configuration files
        
        logger.warning("Cleanup not implemented - manual cleanup recommended")
    
    def run_migration(self) -> bool:
        """Run the complete migration process."""
        logger.info("Starting migration from MCP services to plugin system...")
        
        config = self.load_migration_config()
        dry_run = config.get('dry_run', False)
        
        if dry_run:
            logger.info("Running in DRY RUN mode - no changes will be made")
        
        try:
            # Step 1: Create backup
            if config['migration_steps'].get('backup_configs', True) and not dry_run:
                self.create_backup()
            
            # Step 2: Extract service configurations
            if config['migration_steps'].get('extract_env_vars', True):
                service_configs = self.extract_service_configs()
                if not service_configs:
                    logger.warning("No service configurations found to migrate")
                    return False
            
            # Step 3: Create plugin configurations
            if config['migration_steps'].get('create_plugin_configs', True):
                plugin_configs = self.create_plugin_configs(service_configs)
            
            # Step 4: Save plugin configurations
            if config['migration_steps'].get('migrate_data', True):
                self.save_plugin_configs(plugin_configs, dry_run)
            
            # Step 5: Validate plugins service
            if config['migration_steps'].get('validate_plugins', True):
                if not dry_run and not self.validate_plugins_service():
                    logger.error("Plugins service validation failed")
                    return False
                
                # Load configs via API
                if not self.load_plugin_configs_via_api(plugin_configs, dry_run):
                    logger.error("Failed to load plugin configurations via API")
                    return False
            
            # Step 6: Validate migration
            if not dry_run and not self.validate_migration():
                logger.error("Migration validation failed")
                return False
            
            # Step 7: Cleanup (optional)
            if config['migration_steps'].get('cleanup_old_services', False):
                self.cleanup_old_services(dry_run)
            
            logger.info("Migration completed successfully!")
            return True
            
        except Exception as e:
            logger.error(f"Migration failed: {e}")
            return False

def main():
    parser = argparse.ArgumentParser(description="Migrate from MCP services to plugin system")
    parser.add_argument("--config", help="Migration configuration file", default="migration_config.json")
    parser.add_argument("--dry-run", action="store_true", help="Run in dry-run mode (no changes)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Enable verbose logging")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    migrator = PluginMigrator(args.config)
    
    # Override dry-run setting if specified via command line
    if args.dry_run:
        config = migrator.load_migration_config()
        config['dry_run'] = True
        with open(migrator.config_path, 'w') as f:
            json.dump(config, f, indent=2)
    
    success = migrator.run_migration()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()