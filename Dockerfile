# "Prebuilt image" pattern (see vps-playbook/NEW_APP.md): Kamal always
# builds+pushes whatever `image:` names in config/deploy.yml, so this trivial
# Dockerfile just re-tags upstream's official postgres-flavored image under
# our own ghcr.io/max611 name. Swap this out for a real build off
# apps/server/Dockerfile once Phase 2 custom code lands.
FROM rustrak/rustrak-server:postgres
