import figlet from 'figlet';
import gradient from 'gradient-string';

export const showBanner = (): void => {
    const bannerText = figlet.textSync('CHAKRAVARTI', {
        font: 'ANSI Shadow',
        horizontalLayout: 'default',
        verticalLayout: 'default',
    });

    const paddedBanner = bannerText.split('\n').map(line => '    ' + line).join('\n');

    console.log(gradient.pastel.multiline(paddedBanner));
    console.log(gradient.cristal('      Orchestrating your AI Agents\n'));
};
