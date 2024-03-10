# 20240310.  Learning Rust.  

Poll the WAN device status from a Cradlepoint router.

Work in progress.

Example output:

>                                    NAME TYPE       PLUGGED REASON     SUMMARY
>                           ethernet-sfp0 ethernet   false   Unplugged  unplugged
>                            ethernet-wan ethernet   false   Unplugged  unplugged
>                            mdm-e8a5518d mdm        true    Plugged    configure error
>                            mdm-e8d31846 mdm        true    Plugged    configure error
>           wwan-aa:80:88:35:00:af:2_4G-1 wwan       true    Failover   connected
>
>                                    NAME  STATE           EXCEPTION  TIMEOUT  
>                              WiFiClient  connected       (none)     (none)    
>                                    DHCP  connected       (none)     (none)    
>                           XLATConnector  connected       (none)     (none) 
