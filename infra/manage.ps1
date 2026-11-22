param(
    [Parameter(Mandatory=$true, Position=0)]
    [ValidateSet("deploy", "status", "start", "stop", "ssh", "sync", "bootstrap-status")]
    [string]$Action,
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$ExtraArgs
)

$ErrorActionPreference = "Stop"
$StackName = "mc-rust-dev"
$Region = "us-east-1"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$TemplatePath = Join-Path $PSScriptRoot "template.yaml"
$KeyPath = Join-Path $env:USERPROFILE ".ssh\mc-t4g-key.pem"

function Get-InstanceInfo {
    $out = aws cloudformation describe-stacks --stack-name $StackName --region $Region --query "Stacks[0].Outputs" --output json 2>$null | ConvertFrom-Json
    $id = ($out | Where-Object { $_.OutputKey -eq "InstanceId" }).OutputValue
    if (-not $id) { throw "Stack '$StackName' has no InstanceId output." }
    $inst = aws ec2 describe-instances --instance-ids $id --region $Region --query "Reservations[0].Instances[0].{Id:InstanceId,State:State.Name,IP:PublicIpAddress,Type:InstanceType}" --output json | ConvertFrom-Json
    return $inst
}

switch ($Action) {
    "deploy" {
        aws cloudformation deploy `
            --stack-name $StackName `
            --template-file $TemplatePath `
            --region $Region `
            --no-fail-on-empty-changeset
        $info = Get-InstanceInfo
        Write-Host "Deployed: $($info | ConvertTo-Json -Compress)"
    }
    "status" {
        $info = Get-InstanceInfo
        $info | Format-List
    }
    "start" {
        $info = Get-InstanceInfo
        aws ec2 start-instances --instance-ids $info.Id --region $Region | Out-Null
        aws ec2 wait instance-running --instance-ids $info.Id --region $Region
        $info = Get-InstanceInfo
        Write-Host "Running at $($info.IP)"
    }
    "stop" {
        $info = Get-InstanceInfo
        aws ec2 stop-instances --instance-ids $info.Id --region $Region | Out-Null
        aws ec2 wait instance-stopped --instance-ids $info.Id --region $Region
        Write-Host "Stopped $($info.Id)"
    }
    "ssh" {
        $info = Get-InstanceInfo
        $remoteCmd = if ($ExtraArgs) { $ExtraArgs -join " " } else { "" }
        if ($remoteCmd) {
            ssh -i $KeyPath -o StrictHostKeyChecking=no "ubuntu@$($info.IP)" $remoteCmd
        } else {
            ssh -i $KeyPath -o StrictHostKeyChecking=no "ubuntu@$($info.IP)"
        }
    }
    "bootstrap-status" {
        $info = Get-InstanceInfo
        ssh -i $KeyPath -o StrictHostKeyChecking=no "ubuntu@$($info.IP)" "test -f /var/log/mc-rust-bootstrap.done && echo BOOTSTRAP_DONE || echo BOOTSTRAP_RUNNING; tail -n 20 /var/log/mc-rust-bootstrap.log"
    }
    "sync" {
        $info = Get-InstanceInfo
        $tarPath = Join-Path $env:TEMP "mc-rust-sync.tar.gz"
        Push-Location $RepoRoot
        try {
            git ls-files -co --exclude-standard -z | ForEach-Object { $_ } | Out-Null
            tar --exclude="reference" --exclude="target" --exclude=".git" --exclude="oracle/build" --exclude="oracle/run" -czf $tarPath .
            scp -i $KeyPath -o StrictHostKeyChecking=no $tarPath "ubuntu@$($info.IP):/tmp/mc-rust-sync.tar.gz"
            ssh -i $KeyPath -o StrictHostKeyChecking=no "ubuntu@$($info.IP)" "mkdir -p /opt/mc-rust && tar -xzf /tmp/mc-rust-sync.tar.gz -C /opt/mc-rust && rm -f /tmp/mc-rust-sync.tar.gz"
        } finally {
            Pop-Location
            Remove-Item -Force $tarPath -ErrorAction SilentlyContinue
        }
        Write-Host "Synced workspace to ubuntu@$($info.IP):/opt/mc-rust"
    }
}
