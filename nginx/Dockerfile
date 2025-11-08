# Use the official lightweight nginx image
FROM nginx:alpine

# Copy custom nginx configuration from the build context (./nginx)
COPY nginx/nginx.conf /etc/nginx/nginx.conf

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]