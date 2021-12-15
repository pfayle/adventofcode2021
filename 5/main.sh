terraform init
terraform apply -auto-approve -var part=1 -var inputfile=input.txt
terraform apply -auto-approve -var part=2 -var inputfile=input.txt
