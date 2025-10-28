# Use the official lightweight nginx image
FROM nginx:alpine

# Copy custom nginx configuration. The build context for nginx is the
# `nginx/` directory, so the nginx.conf file is at the root of the context
# (nginx/nginx.conf in the repository). Copy it from the context as nginx.conf
# rather than trying to reference nginx/nginx.conf which would resolve to
# nginx/nginx/nginx.conf inside the context.
COPY nginx.conf /etc/nginx/nginx.conf

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]