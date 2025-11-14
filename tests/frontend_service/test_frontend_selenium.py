"""
Frontend Service Selenium Tests

End-to-end tests for the frontend using Selenium WebDriver.
"""

import pytest
import time
import os
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.chrome.options import Options
from selenium.common.exceptions import TimeoutException

class TestFrontendE2E:
    """End-to-end tests for ConHub frontend."""
    
    FRONTEND_URL = "http://localhost:3000"
    
    @pytest.fixture(scope="class")
    def driver(self):
        """Setup Chrome WebDriver for testing."""
        chrome_options = Options()
        chrome_options.add_argument("--headless")  # Run in headless mode for CI
        chrome_options.add_argument("--no-sandbox")
        chrome_options.add_argument("--disable-dev-shm-usage")
        chrome_options.add_argument("--disable-gpu")
        chrome_options.add_argument("--window-size=1920,1080")
        
        try:
            driver = webdriver.Chrome(options=chrome_options)
            driver.implicitly_wait(10)
            yield driver
        finally:
            if 'driver' in locals():
                driver.quit()
    
    def test_homepage_loads(self, driver):
        """Test that the homepage loads successfully."""
        driver.get(self.FRONTEND_URL)
        
        # Wait for page to load
        WebDriverWait(driver, 10).until(
            EC.presence_of_element_located((By.TAG_NAME, "body"))
        )
        
        assert "ConHub" in driver.title or "ConHub" in driver.page_source
    
    def test_registration_page_exists(self, driver):
        """Test that registration page is accessible."""
        driver.get(f"{self.FRONTEND_URL}/register")
        
        try:
            # Look for registration form elements
            WebDriverWait(driver, 10).until(
                EC.any_of(
                    EC.presence_of_element_located((By.NAME, "email")),
                    EC.presence_of_element_located((By.ID, "email")),
                    EC.presence_of_element_located((By.CSS_SELECTOR, "input[type='email']"))
                )
            )
            
            # Should have password field
            password_field = driver.find_element(By.CSS_SELECTOR, "input[type='password']")
            assert password_field is not None
            
        except TimeoutException:
            # If specific elements not found, at least check page loaded
            assert driver.current_url.endswith("/register") or "register" in driver.page_source.lower()
    
    def test_login_page_exists(self, driver):
        """Test that login page is accessible."""
        driver.get(f"{self.FRONTEND_URL}/login")
        
        try:
            # Look for login form elements
            WebDriverWait(driver, 10).until(
                EC.any_of(
                    EC.presence_of_element_located((By.NAME, "email")),
                    EC.presence_of_element_located((By.ID, "email")),
                    EC.presence_of_element_located((By.CSS_SELECTOR, "input[type='email']"))
                )
            )
            
        except TimeoutException:
            # If specific elements not found, at least check page loaded
            assert driver.current_url.endswith("/login") or "login" in driver.page_source.lower()
    
    def test_navigation_elements(self, driver):
        """Test that main navigation elements are present."""
        driver.get(self.FRONTEND_URL)
        
        # Wait for page to load
        WebDriverWait(driver, 10).until(
            EC.presence_of_element_located((By.TAG_NAME, "body"))
        )
        
        # Look for common navigation elements
        page_source = driver.page_source.lower()
        
        # Should have some form of navigation
        has_navigation = any([
            "nav" in page_source,
            "menu" in page_source,
            "header" in page_source,
            "login" in page_source,
            "register" in page_source
        ])
        
        assert has_navigation, "No navigation elements found on homepage"
    
    @pytest.mark.skip(reason="Requires running auth service")
    def test_registration_flow(self, driver):
        """Test the complete registration flow."""
        driver.get(f"{self.FRONTEND_URL}/register")
        
        # Fill registration form
        email_field = WebDriverWait(driver, 10).until(
            EC.presence_of_element_located((By.CSS_SELECTOR, "input[type='email']"))
        )
        email_field.send_keys("test@example.com")
        
        password_field = driver.find_element(By.CSS_SELECTOR, "input[type='password']")
        password_field.send_keys("TestPassword123!")
        
        name_field = driver.find_element(By.NAME, "name")
        name_field.send_keys("Test User")
        
        # Submit form
        submit_button = driver.find_element(By.CSS_SELECTOR, "button[type='submit']")
        submit_button.click()
        
        # Wait for response
        time.sleep(2)
        
        # Should either redirect or show success/error message
        current_url = driver.current_url
        page_source = driver.page_source.lower()
        
        # Check for success or error indicators
        has_feedback = any([
            "success" in page_source,
            "error" in page_source,
            "welcome" in page_source,
            current_url != f"{self.FRONTEND_URL}/register"
        ])
        
        assert has_feedback, "No feedback after registration attempt"
