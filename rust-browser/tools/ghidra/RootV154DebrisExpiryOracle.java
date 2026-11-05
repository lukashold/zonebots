// Executes original 403450 -> 401000 / 4011a0 / 4015d0 on seeded source records.
// Sound and secondary blast are observed at the call boundary.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.util.HexFormat;
public class RootV154DebrisExpiryOracle extends GhidraScript {
 static final long P=0x5c9a88,A=0x60cfe0,S=0x7fff0000,R=0x12345000;
 long read(EmulatorHelper e,long a,int n){byte[] b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 String hex(EmulatorHelper e,long a,int n){return HexFormat.of().formatHex(e.readMemory(toAddr(a),n));}
 public void run()throws Exception{
  EmulatorHelper e=new EmulatorHelper(currentProgram);
  for(long a:new long[]{R,0x46e660,0x420200})e.setBreakpoint(toAddr(a));
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   for(int k=0;k<144;k++){
    int mode=new int[]{4,2,9}[k/48],s=k%48;
    int px=141,py=99,x=s%12==0?-40:((s%12==1)?-16:346),y=s%12==0?-40:420;
    int code=s/3%3==0?9:(s/3%3==1?42:6),free=s%3==0?-1:s%3==1?0:149;
    int world=s%5,sprite=new int[]{5465,5490,5565,5540,5565}[world],cam=s%2==0?0:1500;
    int cell=(((y&0xfff0)*8)+(x>>4))&65535;
    if(cell<=16383)put(e,0x58aaaaL+cell,1,code);
    byte[] record=new byte[44], ambient=new byte[7200],late=new byte[4*44];
    for(int i=0;i<44;i++)record[i]=(byte)(s*13+i*19);
    for(int i=0;i<7200;i++)ambient[i]=(byte)(s*17+i*19);
    for(int i=0;i<4*44;i++)late[i]=(byte)(s*11+i*17);
    for(int i=0;i<150;i++){int n=i==free?-1:100+i;ambient[i*48]=(byte)n;ambient[i*48+1]=(byte)(n>>8);}
    for(int i=0;i<4;i++){late[i*44]=24;late[i*44+1]=(byte)252;}
    e.writeMemory(toAddr(A),ambient);e.writeMemory(toAddr(P),late);e.writeMemory(toAddr(P+44),record);
    put(e,P+44,2,1);put(e,P+50,2,x-px);put(e,P+52,2,y-py);put(e,P+64,2,mode);put(e,0x4a6520,2,px);put(e,0x4a6530,2,py);
    put(e,0x4d027c,2,sprite);put(e,0x622cb8,2,cam);put(e,0x622cba,2,0);put(e,0x498c24,2,640);put(e,0x498c20,2,600);
    put(e,0x6323a0,1,s/4%2);put(e,0x681588,2,3);put(e,0x4a6e6f,1,7);put(e,0x4a6e72,1,5);
    put(e,0x622e1c,4,4);put(e,0x4d18fa,1,s/8%2);put(e,0x622dc4,1,s/16%2);put(e,0x4d18f2,1,69);
    for(int i=0;i<5;i++)put(e,0x622cdc+i*4,4,0x173510a7L+s*7919L+i*0x9e3779b9L);
    String before=hex(e,P+44,44);
    String preAmbient=free<0?"":hex(e,A+free*48,48);
    String preLate=hex(e,P,4*44);
    put(e,S,4,R);e.writeRegister("ESP",S);e.writeRegister("ECX",P+44);
    if(!e.run(toAddr(0x403450),null,monitor))throw new Exception(e.getLastError());
    int sound=-1,blast=0;boolean retiredAtBlast=false,ownerlessAtBlast=false;
    while(e.readRegister("EIP").longValue()!=R){
     long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
     if(pc==0x46e660){sound=e.readRegister("ECX").intValue();e.writeRegister("EAX",s&1);}
     else if(pc==0x420200){blast++;retiredAtBlast=(short)read(e,P+44,2)==-1000;ownerlessAtBlast=read(e,0x4a6e6f,1)==1&&read(e,0x4a6e72,1)==1&&e.readRegister("ECX").intValue()==0;}
     else throw new Exception("unexpected breakpoint "+Long.toHexString(pc));
     e.writeRegister("EIP",read(e,sp,4));e.writeRegister("ESP",sp+4);
     if(!e.run(monitor))throw new Exception(e.getLastError());
    }
    String after=hex(e,P+44,44),postAmbient=free<0?"":hex(e,A+free*48,48);
    out.printf("{\"case\":%d,\"mode\":%d,\"world\":%d,\"x\":%d,\"y\":%d,\"cell\":%d,\"code\":%d,\"free\":%d,\"camera\":%d,\"soundOn\":%d,\"sound\":%d,\"blast\":%d,\"retiredAtBlast\":%s,\"ownerlessAtBlast\":%s,\"counter\":%d,\"tint\":%d,\"disabled\":%d,\"mp\":%d,\"before\":\"%s\",\"after\":\"%s\",\"ambientBefore\":\"%s\",\"ambientAfter\":\"%s\",\"lateBefore\":\"%s\",\"lateAfter\":\"%s\"}%n",k,mode,world,x,y,cell,code,free,cam,s/4%2,sound,blast,retiredAtBlast,ownerlessAtBlast,read(e,0x681588,2),read(e,0x4d18f2,1),s/8%2,s/16%2,before,after,preAmbient,postAmbient,preLate,hex(e,P,4*44));out.flush();
   }
  }finally{e.dispose();}
 }
}
