docker build -f subnode.Dockerfile -t eightfish-m2-subnode . 
docker build -f subxtproxy.Dockerfile -t eightfish-m2-subxtproxy . 
docker build -f http_gate.Dockerfile -t eightfish-m2-http_gate . 
docker build -f simple_app-a.Dockerfile -t eightfish-m2-simple_app-a . 
docker build -f simple_app-b.Dockerfile -t eightfish-m2-simple_app-b . 
docker build -f simple_app-c.Dockerfile -t eightfish-m2-simple_app-c . 
docker build -f simple_app-d.Dockerfile -t eightfish-m2-simple_app-d . 
