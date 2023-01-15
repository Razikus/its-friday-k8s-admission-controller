# NO MORE DEPLOYMENTS IN FRIDAY

It does what it says. Blocks new deployments in friday on admission controller level.

How often do you break this rule?

How often did you actually DEPLOYED in friday?

Installation 
```
helm install itsfriday https://github.com/Razikus/its-friday-k8s-admission-controller/releases/download/0.1.0/itsfriday-0.1.0.tgz -n itsfriday --create-namespace

```

# Demo

[![asciicast](https://asciinema.org/a/1rndARD0wS2B0AcWAIf1yiKZG.svg)](https://asciinema.org/a/1rndARD0wS2B0AcWAIf1yiKZG)


# Real reason

Real reason is that its very easy to clone this repo, and just replace implementation of controller to do real case - change image, check weather, allow deployments only in nights.

Everything is prepared and not overcomplicated - automatic creation of certificates, automatic webhook registration.



