using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("/")]
public class HelloController : ControllerBase {
    [HttpGet]
    public string Index() {
        return "Hello World";
    }
}
