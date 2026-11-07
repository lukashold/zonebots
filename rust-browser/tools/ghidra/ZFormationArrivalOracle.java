import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;

public class ZFormationArrivalOracle extends ZCoupledFollowerOracle {
    public void run()throws Exception {
        byte[] trig=Files.readAllBytes(Path.of("C:/GOG Games/Z/SINCOS.DAT"));
        try(PrintWriter out=new PrintWriter(getScriptArgs()[0])) {
            for(int s=0;s<512;s++) {
                EmulatorHelper e=new EmulatorHelper(currentProgram);
                try {
                    setup(e,s%128,false,false,trig);
                    long u=U+192, other=U+384;
                    int object=new int[]{0,7,127,128,199}[s%5];
                    long h=0x612f80L+object*44;
                    e.writeMemory(toAddr(0x612f80),new byte[200*44]);
                    for(int r=0;r<21;r++)put(e,R+19+r,1,255);
                    put(e,R+19,1,0);put(e,R+20,1,1);
                    put(e,u+0x36,2,object);put(e,u+0x8a,1,16);put(e,u+0xb5,1,255);
                    put(e,R+50,2,get(e,u+0x2c,2));put(e,R+56,2,get(e,u+0x2e,2));
                    put(e,h,2,(s&4)!=0?-1:400);put(e,h+33,1,(s%8)*5+s/8%5);
                    put(e,other+0x31,1,(s&1)!=0?4:0);put(e,other+0x5f,1,(s&2)!=0?1:0);put(e,other+0xb5,1,object);
                    put(e,0x622d1a,2,0);put(e,0x6323bc,2,1234);
                    if(s<256) {
                        put(e,u+0x14,4,P);put(e,P,2,900);put(e,P+2,2,800);put(e,P+4,4,0);
                        put(e,u+0xc,4,0x5f8428);put(e,0x5f8428,2,700);put(e,0x5f842a,2,600);put(e,0x5f842c,4,0);
                    }
                    call(e,s<256?0x450b30:0x44d570,u);
                    out.printf("{\"scenario\":%d,\"object\":%d,\"rotation\":%d,\"values\":[%s],\"claim\":%d,\"flags8a\":%d,\"secondary\":%d,\"marker\":%d,\"returnAL\":%d}%n",s,object,get(e,h+33,1),fields(e),get(e,u+0xb5,1),get(e,u+0x8a,1),get(e,u+0xc,4),get(e,0x6323bc,2),e.readRegister("EAX").longValue()&255);
                    out.flush();
                }finally{e.dispose();}
            }
        }
    }
}
