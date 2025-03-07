using System.Text.RegularExpressions;

public class Util
{
    public static string HumanizedString(string input)
    {
        return Regex.Replace(Regex.Replace(input, @"([A-Z])", " $1"), $"^ ", "");
    }
}
